#![no_std]
use core::arch::wasm32::unreachable;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { unreachable() }
}

#[repr(C)]
pub struct BlockHeader {
    pub size: u32,
    pub arity: u16,
    pub applied: u16,
}

#[no_mangle]
pub unsafe extern "C" fn get_size(block: *const BlockHeader) -> u32 {
    (*block).size
}

#[no_mangle]
pub unsafe extern "C" fn get_arity(block: *const BlockHeader) -> u16 {
    (*block).arity
}

#[no_mangle]
pub unsafe extern "C" fn apply_nth_arg(block: *mut BlockHeader, arg: u32) {
    let applied = (*block).applied;
    (*block).applied += 1;

    let block_end = block.add(1);
    let arg_location = block_end.cast::<u32>().add(applied as usize);
    arg_location.write(arg)
}
