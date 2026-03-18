#!/usr/bin/env python3
"""Test NVIDIA NIM API connection"""

import os
import requests

API_KEY = os.environ.get("NVIDIA_API_KEY", "")
MODEL = "meta/llama-3.3-70b-instruct"

def test_connection():
    if not API_KEY:
        print("❌ NVIDIA_API_KEY not set")
        print("   Set it with: export NVIDIA_API_KEY='nvapi-...'")
        return False
    
    payload = {
        "model": MODEL,
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Say hello in one word."},
        ],
        "temperature": 0.1,
        "max_tokens": 10,
    }
    
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json",
    }
    
    try:
        response = requests.post(
            "https://integrate.api.nvidia.com/v1/chat/completions",
            headers=headers,
            json=payload,
            timeout=30,
        )
        response.raise_for_status()
        
        result = response.json()
        content = result["choices"][0]["message"]["content"]
        
        print("✅ API connection successful!")
        print(f"   Model: {MODEL}")
        print(f"   Response: {content}")
        return True
        
    except requests.exceptions.HTTPError as e:
        print(f"❌ HTTP Error: {e}")
        if 'response' in dir():
            print(f"   Status: {response.status_code}")
            print(f"   Response: {response.text[:200]}")
        return False
        
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

if __name__ == "__main__":
    success = test_connection()
    exit(0 if success else 1)
