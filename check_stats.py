import os, requests, json

with open('.env', 'r') as f:
    for line in f:
        if line.startswith('MOLTBOOK_API_KEY='):
            api_key = line.strip().split('=', 1)[1]

headers = {
    'Authorization': f'Bearer {api_key}',
    'Content-Type': 'application/json',
}

# Post C
r = requests.get('https://www.moltbook.com/api/v1/posts/198382d9-4f08-463c-816e-d0e29e70ea47', headers=headers, timeout=30)
p = r.json().get('post', {})
upvotes = p.get('upvotes', '?')
comments = p.get('comment_count', '?')
print(f'Post C: {upvotes} upvotes, {comments} comments')

# Post B
r2 = requests.get('https://www.moltbook.com/api/v1/posts/2725be47-42ab-4b55-9143-f68bf73a2088', headers=headers, timeout=30)
p2 = r2.json().get('post', {})
print(f'Post B: {p2.get("upvotes", "?")} upvotes, {p2.get("comment_count", "?")} comments')

# Author info from Post C
author = p.get('author', {})
print(f'Agent: {author.get("name", "?")}')
print(f'Karma: {author.get("karma", "?")}')
print(f'Followers: {author.get("follower_count", "?")}')
