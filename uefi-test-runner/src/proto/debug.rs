use uefi::proto::debug::DebugSupport;
use uefi::table::boot::BootServices;

use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    info!("Running UEFI debug connection protocol test");
    if let Some(debug_support) = bt.find_protocol::<DebugSupport>() {
        let debug_support = unsafe { &mut *debug_support.get() };

        info!("- Architecture: {:?}", debug_support.arch());
    } else {
        warn!("Debug protocol is not supported");
    }
}
