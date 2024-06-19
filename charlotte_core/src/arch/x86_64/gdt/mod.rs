pub mod tss;

use core::{mem::size_of, ptr};

use tss::Tss;

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
struct SegmentDescriptor {
    limit0: u16,
    base0: u16,
    base1: u8,
    access_byte: u8,
    limit1_flags: u8,
    base2: u8,
}

impl SegmentDescriptor {
    fn new() -> Self {
        SegmentDescriptor {
            limit0: 0,
            base0: 0,
            base1: 0,
            access_byte: 0,
            limit1_flags: 0,
            base2: 0,
        }
    }
}

#[repr(C, packed(1))]
struct SystemSegmentDescriptor {
    lower: SegmentDescriptor,
    base3: u32,
    reserved: u32,
}

impl SystemSegmentDescriptor {
    fn new() -> Self {
        SystemSegmentDescriptor {
            lower: SegmentDescriptor::new(),
            base3: 0,
            reserved: 0,
        }
    }
}

#[repr(C, packed(1))]
pub struct Gdt {
    segment_descs: [SegmentDescriptor; 5],
    tss_desc: SystemSegmentDescriptor,
}

impl Gdt {
    pub fn new(tss: &Tss) -> Self {
        let mut gdt = Gdt {
            segment_descs: [SegmentDescriptor::new(); 5],
            tss_desc: SystemSegmentDescriptor::new(),
        };

        //Null descriptor
        gdt.set_segment_desc(0, 0, 0, 0, 0);
        //Kernel Mode Code Segment
        gdt.set_segment_desc(1, 0, 0xFFFFF, 0x9A, 0xA);
        //Kernel Mode Data Segment
        gdt.set_segment_desc(2, 0, 0xFFFFF, 0x92, 0xC);
        //User Mode Code Segment
        gdt.set_segment_desc(3, 0, 0xFFFFF, 0xFA, 0xA);
        //User Mode Data Segment
        gdt.set_segment_desc(4, 0, 0xFFFFF, 0xF2, 0xC);
        //Task State Segment
        gdt.set_tss_desc(ptr::addr_of!(*tss) as u64, size_of::<Tss>() as u32);

        gdt
    }
    fn set_tss_desc(&mut self, base: u64, limit: u32) {
        self.set_segment_desc(5, (base & 0xFFFFFFFF) as u32, limit, 0x89, 0x0);
        self.tss_desc.base3 = ((base & 0xFFFFFFFF00000000) >> 32) as u32;
        self.tss_desc.reserved = 0;
    }
    fn set_segment_desc(
        &mut self,
        index: usize,
        base: u32,
        limit: u32,
        access_byte: u8,
        flags: u8,
    ) {
        let dest_sd = if index == 5 {
            &mut (self.tss_desc.lower)
        } else {
            &mut (self.segment_descs[index])
        };

        dest_sd.limit0 = (limit & 0xFFFF) as u16;
        dest_sd.limit1_flags = ((limit & 0xFF0000) >> 16) as u8; // Only the lower 4 bits of this field encodes limit bits

        dest_sd.base0 = (base & 0xFFFF) as u16;
        dest_sd.base1 = ((base & 0xFF0000) >> 16) as u8;
        dest_sd.base2 = ((base & 0xFF000000) >> 24) as u8;

        dest_sd.access_byte = access_byte;

        dest_sd.limit1_flags |= flags << 4; // The upper 4 bits of this field encodes flags
    }
    pub fn load(&self) {
        unsafe {
            asm_load_gdt(self);
        }
    }
    pub fn reload_segment_regs() {
        unsafe {
            asm_reload_segment_regs();
        }
    }
    pub fn load_tss() {
        unsafe {
            asm_load_tss();
        }
    }
}

extern "C" {
    fn asm_load_gdt(gdt: &Gdt);
    fn asm_reload_segment_regs();
    fn asm_load_tss();
}
