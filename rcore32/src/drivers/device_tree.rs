use device_tree::{DeviceTree, Node};
use device_tree::util::SliceRead;
use core::slice;

struct DtbHeader {
    magic : u32,
    size : u32,
}

pub fn dtb_query_memory(dtb : usize) -> Option<(usize,usize)>{
    let header = unsafe{ &*(dtb as *const DtbHeader) };
    let magic = u32::from_be(header.magic);
    if magic == 0xd00dfeed {
        let size = u32::from_be(header.size); 
        let dtb_data = unsafe { slice::from_raw_parts(dtb as *const u8, size as usize) };

        if let Ok(dt) = DeviceTree::load(dtb_data) {
            if let Some(ret) = query_memory_on_node(&dt.root) {
                return Some(ret);
            }
        }
    }
    None
}

fn query_memory_on_node(dt: &Node) -> Option<(usize,usize)>{
    if let Ok(device_type) = dt.prop_str("device_type") {
        if device_type == "memory" {
            if let Some(reg) = dt.prop_raw("reg") {
                return Option::from((reg.as_slice().read_be_u64(0).unwrap() as usize,
                    reg.as_slice().read_be_u64(8).unwrap() as usize));
            }
        }
    }
    for child in dt.children.iter() {
        if let Some(ret) = query_memory_on_node(child) {
            return Some(ret);
        }
    }
    None
}
