# Changelog
- Added memory module
    - Fixed frame allocator - frame allocator used to return None when `LinkedListNode.size>4096`
    - Fixed global - used to panic regardless. Now it actually works.
