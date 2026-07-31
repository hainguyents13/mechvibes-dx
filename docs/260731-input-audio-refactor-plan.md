# Kế hoạch tái cấu trúc Input/Audio — MechvibesDX

> Ngày: 260731 · Trạng thái: đã duyệt phân tích, chờ triển khai
> Tài liệu tự chứa: người thực hiện không cần ngữ cảnh hội thoại trước đó.

## 1. Bối cảnh

MechvibesDX (Rust + Dioxus 0.7 desktop) là bản viết lại của Mechvibes cũ (Electron + iohook). Bản cũ chạy input hook trong hidden process riêng, trao đổi với UI qua IPC → mượt, không dính UI. Bản DX hiện gom tất cả vào 1 process và đang gặp 4 vấn đề người dùng báo cáo:

1. **Âm thanh không hay bằng bản cũ** (tiếng cụt, khô khi gõ nhanh, click/pop).
2. **Đổi output device không hoạt động, có thể crash UI.**
3. **Focus vào window app thì không bắt được phím** (Windows) — chỉ hoạt động khi unfocus.
4. Yêu cầu giữ đa nền tảng, gồm cả Wayland.

## 2. Hiện trạng kiến trúc (đã khảo sát code)

### 2.1. Luồng input → âm thanh

```
OS threads (rdev / device_query / evdev)
   │  keycode dạng String qua std::mpsc
   ▼
Dioxus UI runtime (src/libs/ui.rs)
   │  use_future + vòng lặp poll try_recv() mỗi 1ms (utils/delay.rs)
   ▼
AudioContext::play_key_event_sound → rodio Sink
```

- Capture đã ở thread riêng, nhưng **trigger âm thanh chạy trong async executor của webview** (`ui.rs:100-153`) → coupling với UI + trễ polling ~1ms.
- `AudioContext` tạo một lần trong `use_hook` (`ui.rs:44`), share qua `Arc` vào 8 component (`use_context`).

### 2.2. Workaround focus trên Windows/X11 ("dual listener")

- `ui.rs:81-88`: bắt `WindowEvent::Focused` (wry) → cờ focus toàn cục (`input_manager.rs:47-67`).
- `input_listener.rs:214,257`: rdev **bỏ qua** keyboard event khi focus = true.
- `focused_input_listener.rs`: khi focus, **polling device_query 100Hz** (10ms) thay thế.
- Nhược điểm: trễ +10ms, nuốt phím gõ nhanh hơn khung poll, cờ focus cập nhật từ UI thread → khoảng mù khi chuyển focus.
- **Nguyên nhân gốc bệnh focus**: KHÔNG phải OS cấm. Hook `WH_KEYBOARD_LL` (rdev) là hook toàn hệ thống, nhận event bất kể focus. Vấn đề là **xung đột trong cùng process** giữa message pump của rdev và event loop tao/wry/WebView2 khi window app ở foreground. ⇒ Tách thread trong cùng process KHÔNG chữa được (đã là thread riêng mà vẫn bị); tách process HOẶC đổi API capture (Raw Input) mới chữa được.

### 2.3. Nguyên nhân âm thanh kém (file:line cụ thể)

| # | Vấn đề | Vị trí |
|---|---|---|
| 1 | Sink lưu `HashMap` key `"KeyA-down"`; gõ lặp nhanh → `insert()` drop sink cũ → **đuôi âm bị chặt cụt** (nguyên nhân chính của cảm giác "khô/cụt") | `src/libs/audio/sound_manager.rs:226` |
| 2 | Cắt PCM cứng tại mốc ms bất kỳ, không fade → **click/pop** | `sound_manager.rs:217` |
| 3 | Vượt `max_voices` (20) → drop sink già nhất ngay → tiếng bị chặt ngang | `sound_manager.rs:249-256` (`manage_active_sinks`) |
| 4 | Rodio resample linear realtime khi rate pack ≠ rate device (kém hơn Web Audio của bản cũ) | (hành vi rodio, load tại `soundpack_loader.rs`) |
| 5 | `AppConfig::load()` **đọc file JSON từ disk mỗi keypress** | `sound_manager.rs:17,261` |

### 2.4. Nguyên nhân switch device hỏng/crash

- Chọn device trong UI **chỉ ghi config** (`device_selector.rs:136`), không có gì áp dụng runtime.
- `AudioContext::create_with_device` (`audio_context.rs:165`) **không được gọi ở đâu** sau khởi động.
- `OutputStream` (rodio/cpal) gắn với thread tạo ra nó, không `Send`, nhưng đang bị `Arc` hoá share khắp component tree → mọi nỗ lực rebuild từ callback UI là nguồn crash.

### 2.5. Tính năng chết: lọc theo thiết bị

- Config đã có `enabled_keyboards`/`enabled_mice` (`state/config.rs`), có `InputDeviceManager` enumerate (winapi: `hidusage`, `setupapi`, `cfgmgr32` đã bật trong Cargo.toml) — nhưng rdev **không phân biệt được event từ thiết bị nào** → tính năng vô dụng trên Windows. (Linux evdev phân biệt được.)

### 2.6. So sánh phương án lớn

| Vấn đề | Fix in-process (engine thread + Raw Input) | Daemon + IPC |
|---|---|---|
| Âm thanh kém | ✅ | ✅ (vẫn phải sửa cùng chỗ) |
| Switch device crash | ✅ | ✅ |
| Bắt phím khi focus | ✅ nếu thay rdev bằng Raw Input | ✅ kể cả giữ rdev |
| Lọc theo thiết bị | ✅ với Raw Input | ✅ với Raw Input |
| Wayland | Như nhau (evdev cần quyền /dev/input dù process nào) | Như nhau |
| UI crash không mất tiếng | ❌ | ✅ |
| Chi phí | Trung bình | Cao (2 binary, IPC, update ×2) |

**Kết luận:** fix in-process (Giai đoạn 1–3) giải quyết toàn bộ vấn đề đã báo cáo. Daemon (Giai đoạn 4) chỉ còn 1 lợi ích riêng, để roadmap.

---

## 3. Kế hoạch triển khai

Mỗi giai đoạn = 1 PR riêng, ship độc lập, giai đoạn sau xây trên giai đoạn trước.

### Giai đoạn 1 — Fix chất lượng âm thanh (1–2 ngày, rủi ro thấp) → v0.4.1

File sửa: `src/libs/audio/sound_manager.rs`, `audio_context.rs`, `soundpack_loader.rs`

1. **Voice pool chồng lấn** (fix quan trọng nhất): bỏ `HashMap<String, Sink>`, thay bằng `Vec<Sink>`; mỗi keypress = voice mới, voice cũ phát nốt tự nhiên. Giữ `key_pressed` map (chống key-repeat) — tách khỏi vòng đời sink.
2. **Khử click/pop**: nhân ramp tuyến tính fade-in ~1–2ms, fade-out ~4–6ms vào đầu/cuối segment trước khi tạo `SamplesBuffer` (không cần dependency mới).
3. **Eviction mềm**: ưu tiên evict sink đã `empty()`; sink đang phát thì fade nhanh (~10ms) rồi drop; cân nhắc tăng `max_voices` 20→32.
4. **Resample 1 lần lúc load pack** về rate của device (crate `rubato` hoặc tự viết), cache theo `(pack_id, device_rate)`.
5. **Bỏ đọc config từ disk mỗi keypress**: cache `enable_sound`/`enable_keyboard_sound`/`enable_mouse_sound` in-memory (atomic/RwLock), UI cập nhật khi đổi.

Nghiệm thu: trill 2 phím 15–20 lần/giây không tiếng nào cụt; pack có reverb dài không click; CPU không tăng đáng kể.

### Giai đoạn 2 — Audio Engine Thread + Switch Device (2–4 ngày, rủi ro trung bình) → v0.5.0

File mới: `src/libs/audio/engine.rs`. File sửa: `audio_context.rs`, `ui.rs`, `main.rs`, `input_manager.rs`, `device_selector.rs`, 8 chỗ `use_context::<Arc<AudioContext>>()`.

1. **Engine thread** (std::thread) vòng `crossbeam_channel::recv()` blocking (crossbeam đã có trong Cargo.toml), sở hữu độc quyền `OutputStream` + samples + key_map + voice pool (biến cục bộ thread, xoá phần lớn `Arc<Mutex>`):
   ```rust
   enum AudioCommand {
       KeyEvent { code: String, down: bool },
       MouseEvent { code: String, down: bool },
       SetVolume(f32), SetMouseVolume(f32),
       LoadSoundpack { id: String, kind: PackKind },
       SwitchDevice(Option<String>), // None = system default
       ToggleEnabled, Shutdown,
   }
   ```
   UI giữ `AudioEngineHandle { tx }` (Clone + Send).
2. **SwitchDevice trong engine thread**: drop voices → drop stream cũ → `OutputStream::try_from_device` trên cùng thread → resample lại cache theo rate mới → tiếng test xác nhận. Thất bại: báo lỗi về UI, giữ stream cũ. `device_selector.rs:136` sau khi ghi config gửi thêm `SwitchDevice` (dây nối đang thiếu). Bonus: cpal error callback → tự fallback default khi rút thiết bị.
3. **Input nối thẳng engine**: listener gửi `AudioCommand` (bỏ String `"UP:KeyA"`). Engine phát xong gửi `UiEvent` qua channel thứ 2 cho UI render hiệu ứng. Xoá 2 vòng polling 1ms `ui.rs:100-153`. Hotkey chuyển vào engine hoặc giữ vòng riêng.
4. `AudioContext` co thành facade UI gọi qua handle.

Nghiệm thu: đổi device khi đang gõ — mượt, không crash; rút tai nghe — tự về default; UI render nặng — tiếng không giật.

### Giai đoạn 3 — Raw Input trên Windows (3–5 ngày, rủi ro trung bình-cao, cần máy Windows thật) → v0.5.0

File mới: `src/libs/rawinput_listener.rs` (Windows-only). Sửa/xoá: `main.rs`, `input_manager.rs` (xoá focus state), `focused_input_listener.rs` (xoá trên Windows), `input_listener.rs` (rdev còn cho macOS + X11 fallback), `ui.rs` (xoá wry focus handler).

1. **Raw Input listener**: thread riêng tạo message-only window (`HWND_MESSAGE`), `RegisterRawInputDevices` + `RIDEV_INPUTSINK` cho keyboard (usage page 0x01/usage 0x06) và mouse (0x01/0x02), vòng `GetMessage` riêng → nhận `WM_INPUT` **bất kể focus** → chữa tận gốc bệnh focus. Winapi features đã đủ trong Cargo.toml. Map scancode/VKey → bộ key code chuẩn hiện tại (`"KeyA"`, `"Digit1"`, ...) để soundpack không đổi. Hotkey Ctrl+Alt+M detect tại đây.
2. **Lọc thiết bị**: `WM_INPUT.hDevice` → `GetRawInputDeviceInfo` lấy device path → đối chiếu `enabled_keyboards`/`enabled_mice`; cache danh sách, cập nhật qua channel khi config đổi.
3. **Dọn dẹp**: xoá trên Windows toàn bộ rdev-keyboard + device_query + focus plumbing (`input_manager.rs:47-67`, `ui.rs:81-88`, skip-logic `input_listener.rs:214,257`).

Ma trận capture sau giai đoạn 3:

| OS | Keyboard | Mouse | Per-device | Bắt phím khi focus |
|---|---|---|---|---|
| Windows | Raw Input | Raw Input | ✅ | ✅ |
| Linux/Wayland | evdev | rdev (nâng evdev sau) | ✅ | ✅ |
| Linux/X11 | evdev ưu tiên, fallback rdev+device_query | rdev | ✅ khi evdev | ✅ khi evdev |
| macOS | rdev (CGEventTap) | rdev | ❌ | cần verify |

Nghiệm thu: focus vào app gõ phím có tiếng, trễ ngang unfocus; disable 1 keyboard trong settings → keyboard đó im; hotkey chạy cả khi focus.

### Giai đoạn 4 — Daemon + IPC (roadmap, sau v0.5, chỉ nếu còn nhu cầu)

- Lợi ích còn lại duy nhất: UI crash không mất tiếng / chạy không cần UI.
- Ranh giới đã sẵn từ giai đoạn 2: serialize `AudioCommand`/`UiEvent` bằng bincode (đã có) qua named pipe (Win) / Unix socket.
- Chi phí: 2 binary, installer, auto-update ×2, vòng đời process (UI tự spawn daemon, single-instance lock).

---

## 4. Thứ tự ship

1. **v0.4.1** = Giai đoạn 1 (trả lời trực tiếp phàn nàn âm thanh).
2. **v0.5.0** = Giai đoạn 2 + 3.
3. Giai đoạn 4 theo feedback.

## 5. Câu hỏi chưa chốt

1. macOS có bị bệnh focus như Windows không? Cần test thật trước khi quyết làm CGEventTap trực tiếp.
2. Resampler: `rubato` (dependency mới, chất lượng cao, chạy 1 lần lúc load — khuyến nghị) hay linear tự viết?
3. X11: chấp nhận yêu cầu user vào group `input` để dùng evdev (như Wayland), hay giữ fallback rdev vĩnh viễn?
4. Giai đoạn 1: tăng `max_voices` 20→32 hay giữ 20 chỉ đổi chiến lược evict?
