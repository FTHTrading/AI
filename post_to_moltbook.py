"""
Genesis Protocol — Moltbook Post Publisher
Posts to Moltbook API with automatic verification challenge handling.
Usage: python post_to_moltbook.py <post_file.json>

Post file format:
{
    "submolt_name": "general",
    "title": "Post Title",
    "content": "Post content..."
}
"""

import json
import os
import re
import sys
import time

import requests

# ── Config ──────────────────────────────────────────────────
def load_api_key():
    """Load API key from .env file."""
    env_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), '.env')
    with open(env_path, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith('MOLTBOOK_API_KEY='):
                return line.split('=', 1)[1].strip()
    raise ValueError("MOLTBOOK_API_KEY not found in .env")

BASE_URL = "https://www.moltbook.com/api/v1"
API_KEY = load_api_key()
HEADERS = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json",
    "User-Agent": "Genesis-Protocol/0.1.0 Moltbot",
}

# ── Verification Challenge Solver ───────────────────────────
def deobfuscate(text):
    """Remove zero-width characters and normalize obfuscated text."""
    # Strip zero-width chars
    zw = re.compile(r'[\u200b\u200c\u200d\u200e\u200f\ufeff\u00ad]')
    text = zw.sub('', text)
    # Collapse repeated letters (e.g., "aaaddddd" -> "ad")
    result = []
    for ch in text:
        if not result or ch != result[-1]:
            result.append(ch)
    text = ''.join(result)
    return text

def solve_math(text):
    """Extract and solve a simple math challenge from text."""
    text = deobfuscate(text)
    # Find numbers
    numbers = re.findall(r'[\d,]+\.?\d*', text)
    nums = []
    for n in numbers:
        n = n.replace(',', '')
        nums.append(float(n))
    
    if len(nums) < 2:
        return None
    
    # Detect operation
    text_lower = text.lower()
    if any(w in text_lower for w in ['add', 'plus', 'sum', '+']):
        result = nums[0] + nums[1]
    elif any(w in text_lower for w in ['subtract', 'minus', 'difference', '-']):
        result = nums[0] - nums[1]
    elif any(w in text_lower for w in ['multiply', 'times', 'product', '*', '×']):
        result = nums[0] * nums[1]
    elif any(w in text_lower for w in ['divide', 'quotient', '/', '÷']):
        if nums[1] != 0:
            result = nums[0] / nums[1]
        else:
            return None
    else:
        # Default to addition
        result = nums[0] + nums[1]
    
    # Return as int if whole number
    if result == int(result):
        return str(int(result))
    return f"{result:.2f}"

def handle_challenge(resp_json):
    """Handle a Moltbook verification challenge."""
    challenge = resp_json.get('challenge_text', '') or resp_json.get('challenge', '')
    code = resp_json.get('verification_code', '') or resp_json.get('code', '')
    
    if not challenge or not code:
        print(f"  [!] Challenge received but missing fields: {list(resp_json.keys())}")
        return False
    
    print(f"  [*] Verification challenge: {challenge[:80]}...")
    print(f"  [*] Code: {code}")
    
    answer = solve_math(challenge)
    if not answer:
        print(f"  [!] Could not solve challenge")
        return False
    
    print(f"  [*] Answer: {answer}")
    
    verify_url = f"{BASE_URL}/verify"
    payload = {"verification_code": code, "answer": answer}
    
    r = requests.post(verify_url, json=payload, headers=HEADERS, timeout=15)
    print(f"  [*] Verify response: {r.status_code}")
    
    if r.status_code in (200, 201):
        print(f"  [+] Verification passed!")
        return True
    else:
        print(f"  [!] Verification failed: {r.text[:200]}")
        return False

# ── Post Publisher ──────────────────────────────────────────
def post(submolt, title, content, max_retries=3):
    """Publish a post to Moltbook with retry and challenge handling."""
    url = f"{BASE_URL}/posts"
    payload = {
        "submolt_name": submolt,
        "title": title,
        "content": content,
    }
    
    print(f"\n{'='*60}")
    print(f"  Posting to m/{submolt}")
    print(f"  Title: {title}")
    print(f"  Content length: {len(content)} chars")
    print(f"{'='*60}")
    
    for attempt in range(max_retries):
        try:
            r = requests.post(url, json=payload, headers=HEADERS, timeout=30)
            print(f"\n  [*] Attempt {attempt+1}: HTTP {r.status_code}")
            
            if r.status_code in (200, 201):
                body = r.json() if r.headers.get('content-type', '').startswith('application/json') else {}
                
                # Check for verification challenge
                if 'challenge_text' in body or 'challenge' in body or 'verification_code' in body:
                    return handle_challenge(body)
                
                print(f"  [+] Post published successfully!")
                if 'post' in body:
                    post_data = body['post']
                    print(f"  [+] Post ID: {post_data.get('id', 'N/A')}")
                    print(f"  [+] URL: {post_data.get('url', 'N/A')}")
                return True
            
            elif r.status_code == 403:
                body_text = r.text
                if 'suspended' in body_text.lower():
                    print(f"  [!] Agent suspended — cannot post")
                    return False
                # Try parsing as challenge
                try:
                    body = r.json()
                    if 'challenge_text' in body or 'verification_code' in body:
                        return handle_challenge(body)
                except:
                    pass
                print(f"  [!] 403 Forbidden: {body_text[:200]}")
                return False
            
            elif r.status_code == 429:
                print(f"  [!] Rate limited (429) — waiting 60s")
                time.sleep(60)
            
            else:
                print(f"  [!] Rejected: {r.text[:300]}")
        
        except Exception as e:
            print(f"  [!] Error: {e}")
        
        if attempt < max_retries - 1:
            delay = 0.2 * (2 ** attempt)
            print(f"  [*] Retrying in {delay:.1f}s...")
            time.sleep(delay)
    
    print(f"  [!] Failed after {max_retries} attempts")
    return False

# ── Main ────────────────────────────────────────────────────
if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python post_to_moltbook.py <post_file.json>")
        print("       python post_to_moltbook.py --inline <submolt> <title> <content>")
        sys.exit(1)
    
    if sys.argv[1] == '--inline':
        if len(sys.argv) < 5:
            print("Usage: python post_to_moltbook.py --inline <submolt> <title> <content>")
            sys.exit(1)
        submolt = sys.argv[2]
        title = sys.argv[3]
        content = sys.argv[4]
    else:
        with open(sys.argv[1], 'r', encoding='utf-8') as f:
            data = json.load(f)
        submolt = data['submolt_name']
        title = data['title']
        content = data['content']
    
    success = post(submolt, title, content)
    sys.exit(0 if success else 1)
