#![no_std]
use core::arch::wasm32::unreachable;
use core::mem;
use core::panic::PanicInfo;
use core::ptr;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable() }
}

#[repr(C)]
pub struct BlockHeader {
    pub size: usize,
    // TODO: Move this information into the free space for unallocated blocks
    pub next: *const BlockHeader,
}

/// Let's work on a bump allocator
#[no_mangle]
static mut WATERMARK: u32 = 4;

#[no_mangle]
static mut HEAD: *const BlockHeader = ptr::null();

fn align(n: usize) -> usize {
    (n + mem::size_of::<usize>() - 1) & !(mem::size_of::<usize>() - 1)
}

#[no_mangle]
pub unsafe extern "C" fn alloc(size: usize) -> *const u8 {
    let size = align(size + mem::size_of::<BlockHeader>());
    match freelist_alloc(size) {
        None => bump_alloc(size),
        Some(ptr) => ptr,
    }
}

unsafe fn bump_alloc(size: usize) -> *const u8 {
    let res = WATERMARK;
    WATERMARK += size as u32;
    let res = res as *mut BlockHeader;
    res.write(BlockHeader {
        size,
        next: ptr::null(),
    });
    res.add(1) as *const u8
}

unsafe fn freelist_alloc(size: usize) -> Option<*const u8> {
    let mut current_block = HEAD;
    if current_block.is_null() {
        None
    } else if (*current_block).size >= size {
        HEAD = (*current_block).next;
        let current_block = current_block as *mut BlockHeader;
        (*current_block).next = ptr::null();
        Some(current_block.add(1) as *const u8)
    } else {
        loop {
            let block = (*current_block).next;
            if block.is_null() {
                break None;
            }
            if (*block).size >= size {
                let current_block = current_block as *mut BlockHeader;
                let block = block as *mut BlockHeader;
                (*current_block).next = (*block).next;
                (*block).next = ptr::null();
                break Some(block.add(1) as *const u8);
            } else {
                current_block = (*block).next
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *const u8) {
    let header_ptr = (ptr as *mut BlockHeader).offset(-1);
    if HEAD.is_null() {
        HEAD = header_ptr;
    } else {
        (*header_ptr).next = HEAD;
        HEAD = header_ptr;
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_length() -> u32 {
    let mut res = 0;
    let mut current = HEAD;
    loop {
        if current.is_null() {
            break;
        } else {
            res += 1;
            current = (*current).next;
        }
    }
    res
}

#[no_mangle]
pub unsafe extern "C" fn mymain() -> u32 {
    let my_ptr = alloc(10);
    free(my_ptr);
    let my_ptr = alloc(20);
    free(my_ptr);

    let _my_next_ptr = alloc(10);
    let _my_next_ptr = alloc(20);
    free_length()
}
