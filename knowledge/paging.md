# pseudocode to explain virtual address to physical address translation
```
structure PageTableEntry: u64 {
    present: Bits(0)
    writeable: Bits(1)
    userAccessible: Bits(2)
    writeThroughCache: Bits(3)
    disableCache: Bits(4)
    accessed: Bits(5)
    dirty: Bits(6)
    hugePage: Bits(7)   // NOTE: muse be 0 in level 1 (directory) and 4 (last index)
    osDataA: Bits(9..=11)
    pageTableAddr: Bits(12..=51)
    osDataB: Bits(52..=62)
    noExecute: Bits(63)
}
structure PageTable: [u64;512]

function translate(addr:VirtualAddress)->Pointer:
    pageDirectory: Pointer(PageTable) = Reg(cr3)
    
    level2PageTableIndex: u9 = ((addr>>39)&511)
    level2PageTable: PageTableEntry = pageDirectory[level2PageTableIndex]

    level3PageTableIndex: u9 = ((addr>>30)&511)
    level3PageTable: PageTableEntry = level2PageTable[level3PageTableIndex]

    level4PageTableIndex: u9 = ((addr>>21)&511)
    level4PageTable: PageTableEntry = level3PageTable[level4PageTableIndex]

    pageIndex: u9 = ((addr>>12)&511)
    page: PageTableEntry = level4PageTable[pageIndex]

    pointer: Pointer = ((page.pageTableAddr as u64)&0xfff)|(addr&0xfff)
    return pointer
```
