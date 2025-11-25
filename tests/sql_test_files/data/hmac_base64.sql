select '97yD9DBThCSxMpjmqm+xQ+9NWaFJRhdZl0edvC0aPNg=' as expected,
    sqlpage.hmac('The quick brown fox jumps over the lazy dog', 'key', 'sha256-base64') as actual;