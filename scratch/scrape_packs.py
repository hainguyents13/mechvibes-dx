import urllib.request
import re
import json
import time
import os

def main():
    print("🕸️ Starting Mechvibes soundpacks scraper...")
    url = "https://mechvibes.com/sound-packs/"
    
    try:
        req = urllib.request.Request(
            url, 
            headers={'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'}
        )
        with urllib.request.urlopen(req) as response:
            html = response.read().decode('utf-8')
    except Exception as e:
        print(f"❌ Failed to fetch main page: {e}")
        return

    # Find all soundpack subpage links
    # Matches href="/sound-packs/some-pack-id" or https://mechvibes.com/sound-packs/some-pack-id
    matches = re.findall(r'href=["\'](https://mechvibes.com/sound-packs/[a-zA-Z0-9_\-]+)/?["\']', html)
    matches += re.findall(r'href=["\'](/sound-packs/[a-zA-Z0-9_\-]+)/?["\']', html)
    
    # Normalize and deduplicate
    subpages = set()
    for match in matches:
        if not match.endswith('/sound-packs') and not match.endswith('/sound-packs/'):
            if match.startswith('/'):
                subpages.add("https://mechvibes.com" + match)
            else:
                subpages.add(match)
                
    subpages = sorted(list(subpages))
    print(f"🔍 Found {len(subpages)} soundpack subpages. Scraping details...")

    soundpacks = []
    
    for i, subpage_url in enumerate(subpages):
        print(f"({i+1}/{len(subpages)}) Fetching {subpage_url}...")
        try:
            req = urllib.request.Request(
                subpage_url, 
                headers={'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'}
            )
            with urllib.request.urlopen(req) as response:
                sub_html = response.read().decode('utf-8')
        except Exception as e:
            print(f"  ⚠️ Failed: {e}")
            time.sleep(0.5)
            continue
            
        # Extract title/name
        # Match <title>CherryMX Black - ABS keycaps - Mechvibes</title> or similar
        title_match = re.search(r'<title>(.*?)</title>', sub_html, re.IGNORECASE)
        name = "Unknown Pack"
        if title_match:
            title_text = title_match.group(1).strip()
            # Clean up suffix "- Mechvibes"
            if " - Mechvibes" in title_text:
                name = title_text.split(" - Mechvibes")[0].strip()
            elif " | Mechvibes" in title_text:
                name = title_text.split(" | Mechvibes")[0].strip()
            else:
                name = title_text
                
        # Also look for <h2> tag which often contains the clean name
        h2_match = re.search(r'<h2[^>]*>(.*?)</h2>', sub_html, re.IGNORECASE)
        if h2_match:
            name = h2_match.group(1).strip()

        # Extract zip URL
        # Matches href=".../dist/....zip"
        zip_match = re.search(r'href=["\'](https://mechvibes.com/[^"\']+\.zip)["\']', sub_html)
        if not zip_match:
            zip_match = re.search(r'href=["\'](/[^"\']+\.zip)["\']', sub_html)
            
        if zip_match:
            zip_url = zip_match.group(1)
            if zip_url.startswith('/'):
                zip_url = "https://mechvibes.com" + zip_url
                
            pack_id = subpage_url.split('/')[-1]
            
            # Determine keyboard vs mouse based on tag or content (most are keyboard)
            pack_type = "Keyboard"
            if "mouse" in name.lower() or "click" in name.lower() or "scroll" in name.lower():
                pack_type = "Mouse"
                
            soundpacks.append({
                "id": pack_id,
                "name": name,
                "download_url": zip_url,
                "learn_more_url": subpage_url,
                "type": pack_type
            })
            print(f"  ✅ Extracted: '{name}' -> {zip_url}")
        else:
            print(f"  ⚠️ No zip file found for {name}")
            
        time.sleep(0.1) # Be a good citizen

    # Ensure assets directory exists
    os.makedirs("assets", exist_ok=True)
    with open("assets/online_soundpacks.json", "w", encoding="utf-8") as f:
        json.dump(soundpacks, f, indent=2, ensure_ascii=False)
        
    print(f"🎉 Scraping complete! Saved {len(soundpacks)} soundpacks to assets/online_soundpacks.json")

if __name__ == "__main__":
    main()
