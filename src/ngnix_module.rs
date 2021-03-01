use nginx::*;
use std::os::raw::{c_char, c_void};

#[no_mangle]
static mut commands: [ngx_command_t; 1] = [ngx_command_t {
    name: ngx_str_t {
        len: 0,
        data: std::ptr::null_mut(),
    },
    type_: 0,
    set: None,
    conf: 0,
    offset: 0,
    post: std::ptr::null_mut(),
}];

#[no_mangle]
static ngx_open_rusty_mod_ctx: ngx_http_module_t = ngx_http_module_t {
    postconfiguration: Some(ngx_http_mod_init),
    preconfiguration: None,
    create_loc_conf: None,
    merge_loc_conf: None,
    merge_srv_conf: None,
    init_main_conf: None,
    create_main_conf: None,
    create_srv_conf: None,
};

pub const NGX_RS_MODULE_SIGNATURE: &'static [u8; 41usize] =
    b"8,4,8,0000111111010111001110101111000110\0";

#[no_mangle]
pub static mut ngx_open_rusty_mod: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::max_value(),
    index: ngx_uint_t::max_value(),
    name: std::ptr::null_mut(),
    version: nginx_version as ngx_uint_t,
    signature: NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,
    ctx: &ngx_open_rusty_mod_ctx as *const _ as *mut _,
    commands: unsafe { &commands[0] as *const _ as *mut _ },
    type_: NGX_HTTP_MODULE as ngx_uint_t,
    init_master: None,
    init_module: None,
    init_process: None,
    init_thread: None,
    exit_thread: None,
    exit_process: None,
    exit_master: None,
    spare0: 0,
    spare1: 0,
    spare_hook0: 0,
    spare_hook1: 0,
    spare_hook2: 0,
    spare_hook3: 0,
    spare_hook4: 0,
    spare_hook5: 0,
    spare_hook6: 0,
    spare_hook7: 0,
};

pub unsafe fn ngx_http_conf_get_module_main_conf(
    cf: *mut ngx_conf_t,
    module: &ngx_module_t,
) -> *mut c_void {
    let http_conf_ctx = (*cf).ctx as *mut ngx_http_conf_ctx_t;
    *(*http_conf_ctx).main_conf.add(module.ctx_index)
}

#[no_mangle]
pub unsafe extern "C" fn ngx_http_mod_init(cf: *mut ngx_conf_t) -> ngx_int_t {
    let cmcf = ngx_http_conf_get_module_main_conf(cf, &ngx_http_core_module)
        as *mut ngx_http_core_main_conf_t;

    let h = ngx_array_push(
        &mut (*cmcf).phases[ngx_http_phases_NGX_HTTP_ACCESS_PHASE as usize].handlers,
    ) as *mut ngx_http_handler_pt;
    if h.is_null() {
        return NGX_ERROR as isize;
    }
    *h = Some(request_handler);

    return NGX_OK as isize;
}

#[no_mangle]
unsafe extern "C" fn request_handler(r: *mut ngx_http_request_t) -> ngx_int_t {
    for header in (*r).headers_in.into_iter() {
        if header.key() == "Test-Header3".to_string() {
            return NGX_HTTP_FORBIDDEN as isize;
        }
    }
    return NGX_DECLINED as isize;
}
