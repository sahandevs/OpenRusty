use crate::{
    ngnix_utils::*,
    ngx_string,
    script::{EvalContext, ScriptEngine},
};
use nginx::*;
use rhai::{Dynamic, ImmutableString};
use std::{
    borrow::Borrow,
    collections::HashMap,
    os::raw::{c_char, c_void},
    ptr,
};
pub const NGX_RS_MODULE_SIGNATURE: &'static [u8; 41usize] =
    b"8,4,8,0000111111010111001110101111000110\0";

#[no_mangle]
static mut commands: [ngx_command_t; 2] = [
    ngx_command_t {
        name: ngx_string!("open_rusty_request_filter\0"),
        type_: (NGX_HTTP_LOC_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_open_rusty_request_filter_set),
        conf: 16,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_command_t {
        name: ngx_str_t {
            len: 0,
            data: std::ptr::null_mut(),
        },
        type_: 0,
        set: None,
        conf: 0,
        offset: 0,
        post: std::ptr::null_mut(),
    },
];

#[no_mangle]
static ngx_open_rusty_mod_ctx: ngx_http_module_t = ngx_http_module_t {
    postconfiguration: None,
    preconfiguration: None,
    create_loc_conf: Some(create_loc_conf),
    merge_loc_conf: Some(merge_loc_conf),
    merge_srv_conf: None,
    init_main_conf: None,
    create_main_conf: None,
    create_srv_conf: None,
};

struct LocConf {
    script_engine: Option<ScriptEngine>,
}

unsafe extern "C" fn create_loc_conf(cf: *mut ngx_conf_t) -> *mut c_void {
    let mut pool = Pool::from_ngx_pool((*cf).pool);
    pool.allocate::<LocConf>(LocConf {
        script_engine: None,
    }) as *mut c_void
}

unsafe extern "C" fn merge_loc_conf(
    _cf: *mut ngx_conf_t,
    _prev: *mut c_void,
    _conf: *mut c_void,
) -> *mut c_char {
    // TODO: support merge
    // let prev = &mut *(prev as *mut LocConf);
    // let conf = &mut *(conf as *mut LocConf);
    // if conf.script_engine.is_none() {
    //     conf.script_engine = prev.script_engine;
    // }
    ptr::null_mut()
}

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

pub unsafe fn ngx_http_conf_get_module_loc_conf(
    cf: *mut ngx_conf_t,
    module: &ngx_module_t,
) -> *mut c_void {
    let http_conf_ctx = (*cf).ctx as *mut ngx_http_conf_ctx_t;
    *(*http_conf_ctx).loc_conf.add(module.ctx_index)
}

#[no_mangle]
unsafe extern "C" fn ngx_open_rusty_request_filter_set(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    let conf = &mut *(conf as *mut LocConf);
    let args = (*(*cf).args).elts as *mut ngx_str_t;
    let value = NgxStr::from_ngx_str(*args.add(1));

    let script = String::from(value.to_string_lossy());
    conf.script_engine = Some(ScriptEngine::new(&script));

    let clcf = ngx_http_conf_get_module_loc_conf(cf, &ngx_http_core_module)
        as *mut ngx_http_core_loc_conf_t;

    (*clcf).handler = Some(open_rusty_request_filter_handler);
    ptr::null_mut()
}

pub fn get_module_loc_conf(r: *mut ngx_http_request_t, module: &ngx_module_t) -> *mut c_void {
    unsafe { *(*r).loc_conf.add(module.ctx_index) }
}

#[no_mangle]
unsafe extern "C" fn open_rusty_request_filter_handler(r: *mut ngx_http_request_t) -> ngx_int_t {
    let hlcf = get_module_loc_conf(r, &ngx_open_rusty_mod) as *mut LocConf;
    let engine = (*hlcf).borrow();
    let engine = *&engine.script_engine.as_ref().unwrap();

    let mut headers: HashMap<ImmutableString, Dynamic> = HashMap::new();

    for header in (*r).headers_in.into_iter() {
        headers.insert(header.key().into(), header.value().into());
    }

    let ctx = EvalContext { headers };

    let script_result = engine.run(&ctx);

    match script_result {
        Some(x) => x as isize,
        _ => NGX_DECLINED as isize,
    }
}
