#![allow(non_camel_case_types)]
#![allow(dead_code)]

#[cfg(all(unix, not(target_os = "macos")))]
const FILE: &str = "libpcap.so";
#[cfg(target_os = "macos")]
const FILE: &str = "libpcap.dylib";
#[cfg(windows)]
const FILE: &str = "wpcap.dll";

// Example usage:
crate::generate_bindings! {
    FILE,
    PcapInner,

    // fn pcap_create(arg1: *const c_char, arg2: *mut c_char) -> *mut pcap_t;
    // fn pcap_set_snaplen(arg1: *mut pcap_t, arg2: c_int) -> c_int;
    // fn pcap_set_promisc(arg1: *mut pcap_t, arg2: c_int) -> c_int;
    // fn pcap_can_set_rfmon(arg1: *mut pcap_t) -> c_int;
    // fn pcap_set_timeout(arg1: *mut pcap_t, arg2: c_int) -> c_int;
    // fn pcap_set_buffer_size(arg1: *mut pcap_t, arg2: c_int) -> c_int;
    // fn pcap_activate(arg1: *mut pcap_t) -> c_int;
    fn pcap_open_live(arg1: *const c_char, arg2: c_int, arg3: c_int, arg4: c_int, arg5: *mut c_char) -> *mut pcap_t;
    // fn pcap_open_dead(arg1: c_int, arg2: c_int) -> *mut pcap_t;
    // fn pcap_open_offline(arg1: *const c_char, arg2: *mut c_char) -> *mut pcap_t;
    // fn pcap_fopen_offline(arg1: *mut FILE, arg2: *mut c_char) -> *mut pcap_t;
    fn pcap_close(arg1: *mut pcap_t);
    fn pcap_loop(arg1: *mut pcap_t,arg2: c_int, arg3: pcap_handler, arg4: *mut c_uchar) -> c_int;
    // fn pcap_dispatch(arg1: *mut pcap_t, arg2: c_int, arg3: pcap_handler,arg4: *mut c_uchar)-> c_int;
    // fn pcap_next(arg1: *mut pcap_t, arg2: *mut pcap_pkthdr) -> *const c_uchar;
    // fn pcap_next_ex(arg1: *mut pcap_t,arg2: *mut *mut pcap_pkthdr, arg3: *mut *const c_uchar) -> c_int;
    fn pcap_breakloop(arg1: *mut pcap_t);
    // fn pcap_stats(arg1: *mut pcap_t, arg2: *mut pcap_stat) -> c_int;
    // fn pcap_setfilter(arg1: *mut pcap_t, arg2: *mut bpf_program) -> c_int;
    // fn pcap_setdirection(arg1: *mut pcap_t, arg2: pcap_direction_t) -> c_int;
    // fn pcap_getnonblock(arg1: *mut pcap_t, arg2: *mut c_char) -> c_int;
    // fn pcap_setnonblock(arg1: *mut pcap_t, arg2: c_int, arg3: *mut c_char) -> c_int;
    // fn pcap_sendpacket(arg1: *mut pcap_t, arg2: *const c_uchar, arg3: c_int) -> c_int;
    // fn pcap_statustostr(arg1: c_int) -> *const c_char;
    // fn pcap_strerror(arg1: c_int) -> *const c_char;
    // fn pcap_geterr(arg1: *mut pcap_t) -> *mut c_char;
    // fn pcap_perror(arg1: *mut pcap_t, arg2: *mut c_char);
    // fn pcap_compile(arg1: *mut pcap_t,arg2: *mut bpf_program,arg3: *const c_char,arg4: c_int, arg5: c_uint) -> c_int;
    // fn pcap_compile_nopcap(arg1: c_int, arg2: c_int, arg3: *mut bpf_program, arg4: *const c_char, arg5: c_int, arg6: c_uint) -> c_int;
    // fn pcap_freecode(arg1: *mut bpf_program);
    // fn pcap_offline_filter(arg1: *const bpf_program,arg2: *const pcap_pkthdr,arg3: *const c_uchar) -> c_int;
    // fn pcap_datalink(arg1: *mut pcap_t) -> c_int;
    // fn pcap_datalink_ext(arg1: *mut pcap_t) -> c_int;
    // fn pcap_list_datalinks(arg1: *mut pcap_t, arg2: *mut *mut c_int) -> c_int;
    // fn pcap_set_datalink(arg1: *mut pcap_t, arg2: c_int) -> c_int;
    // fn pcap_free_datalinks(arg1: *mut c_int);
    // fn pcap_datalink_name_to_val(arg1: *const c_char) -> c_int;
    // fn pcap_datalink_val_to_name(arg1: c_int) -> *const c_char;
    // fn pcap_datalink_val_to_description(arg1: c_int) -> *const c_char;
    // fn pcap_snapshot(arg1: *mut pcap_t) -> c_int;
    // fn pcap_is_swapped(arg1: *mut pcap_t) -> c_int;
    // fn pcap_major_version(arg1: *mut pcap_t) -> c_int;
    // fn pcap_minor_version(arg1: *mut pcap_t) -> c_int;
    // fn pcap_file(arg1: *mut pcap_t) -> *mut FILE;
    // fn pcap_fileno(arg1: *mut pcap_t) -> c_int;
    // fn pcap_dump_open(arg1: *mut pcap_t, arg2: *const c_char) -> *mut pcap_dumper_t;
    // fn pcap_dump_fopen(arg1: *mut pcap_t, fp: *mut FILE) -> *mut pcap_dumper_t;
    // fn pcap_dump_file(arg1: *mut pcap_dumper_t) -> *mut FILE;
    // fn pcap_dump_ftell(arg1: *mut pcap_dumper_t) -> c_long;
    // fn pcap_dump_flush(arg1: *mut pcap_dumper_t) -> c_int;
    // fn pcap_dump_close(arg1: *mut pcap_dumper_t);
    // fn pcap_dump(arg1: *mut c_uchar, arg2: *const pcap_pkthdr, arg3: *const c_uchar);
    fn pcap_findalldevs(arg1: *mut *mut pcap_if_t, arg2: *mut c_char) -> c_int;
    fn pcap_freealldevs(arg1: *mut pcap_if_t);
    fn pcap_lib_version() -> *const c_char;
    // fn bpf_image(arg1: *const bpf_insn, arg2: c_int) -> *mut c_char;
    // fn bpf_dump(arg1: *const bpf_program, arg2: c_int);
    // fn pcap_get_selectable_fd(arg1: *mut pcap_t) -> c_int;
}

// typedef void (*pcap_handler)(u_char *user, const struct pcap_pkthdr *h, const u_char *bytes);
pub type pcap_handler =
    extern "C" fn(slf: *mut c_uchar, header: *const pcap_pkthdr, packet: *const c_uchar);

#[derive(Copy, Clone)]
pub enum pcap_t {}

#[derive(Copy, Clone)]
pub(crate) struct PcapTSend(pub *mut pcap_t);
unsafe impl Send for PcapTSend {}
unsafe impl Sync for PcapTSend {}

pub const PCAP_IF_LOOPBACK: u32 = 0x00000001;
pub const PCAP_IF_UP: u32 = 0x00000002;
pub const PCAP_IF_RUNNING: u32 = 0x00000004;
pub const PCAP_IF_WIRELESS: u32 = 0x00000008;
pub const PCAP_IF_CONNECTION_STATUS: u32 = 0x00000030;
pub const PCAP_IF_CONNECTION_STATUS_UNKNOWN: u32 = 0x00000000;
pub const PCAP_IF_CONNECTION_STATUS_CONNECTED: u32 = 0x00000010;
pub const PCAP_IF_CONNECTION_STATUS_DISCONNECTED: u32 = 0x00000020;
pub const PCAP_IF_CONNECTION_STATUS_NOT_APPLICABLE: u32 = 0x00000030;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct pcap_pkthdr {
    pub ts: timeval,
    pub caplen: c_uint,
    pub len: c_uint,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct pcap_if_t {
    pub next: *mut pcap_if_t,
    pub name: *mut c_char,
    pub description: *mut c_char,
    pub addresses: *mut pcap_addr_t,
    pub flags: c_uint,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct pcap_addr_t {
    pub next: *mut pcap_addr_t,
    pub addr: *mut sockaddr,
    pub netmask: *mut sockaddr,
    pub broadaddr: *mut sockaddr,
    pub dstaddr: *mut sockaddr,
}
