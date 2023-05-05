use reqwest::StatusCode;
use crate::API::objects::VapiHost;
mod api_impl;
mod objects;


pub fn new_api(hostname : String) -> api_impl::VcenterApi {
    return api_impl::VcenterApi::new(hostname);
}

pub fn get_all_hosts(api : api_impl::VcenterApi,session : String) -> Vec<VapiHost>{
    let hosts = api_impl::VcenterApi::get_all_hosts(&api,session);
    let v2: serde_json::Value = serde_json::from_str(&hosts).unwrap();
    let host_list: Vec<VapiHost> = serde_json::from_value(v2["value"].clone()).unwrap();
    return host_list;
}

pub fn get_host(api : api_impl::VcenterApi,session : String, name : String) -> VapiHost{
    let host = api_impl::VcenterApi::get_host(&api,session, name);
    let v2: serde_json::Value = serde_json::from_str(&host).unwrap();
    let host: Vec<VapiHost> = serde_json::from_value(v2["value"].clone()).unwrap();
    return host[0].clone();
}

pub fn get_all_vms(api : api_impl::VcenterApi,session : String) -> Vec<objects::VapiVm> {
    let vms = api_impl::VcenterApi::get_all_vms(&api,session);
    let v2: serde_json::Value = serde_json::from_str(&vms).unwrap();
    let vm_list: Vec<objects::VapiVm> = serde_json::from_value(v2["value"].clone()).unwrap();
    return vm_list;
}

pub fn get_vms_from_host(api : api_impl::VcenterApi,session: String, host: String) -> Vec<objects::VapiVm> {
    let sessiontmp = session.clone();
    let host = api_impl::VcenterApi::get_host(&api,session, host);
    let hostjson: serde_json::Value = serde_json::from_str(&host).unwrap();
    let host: Vec<VapiHost> = serde_json::from_value(hostjson["value"].clone()).unwrap();



    let vms = api_impl::VcenterApi::get_vms_from_host(&api,sessiontmp, host[0].host.clone());
    let v2: serde_json::Value = serde_json::from_str(&vms).unwrap();
    let vm_list: Vec<objects::VapiVm> = serde_json::from_value(v2["value"].clone()).unwrap();
    return vm_list;
}

pub fn shutdown_vm(api : api_impl::VcenterApi,session : String, vm : String) -> StatusCode  {
    return api_impl::VcenterApi::shutdown_vm(&api,session, vm);
}

pub fn poweron_vm(api : api_impl::VcenterApi,session : String, vm : String) -> StatusCode  {
    return api_impl::VcenterApi::start_vm(&api,session, vm);
}

pub fn reboot_vm(api : api_impl::VcenterApi,session : String, vm : String) -> StatusCode  {
    return api_impl::VcenterApi::reboot_vm(&api,session, vm);
}

pub fn authenticate(api : api_impl::VcenterApi,username : String,password : String, host : String) -> String {
    let session = api_impl::VcenterApi::get_session(&api
                                                    ,username
                                                  , password
                                                  , host);
    let v: serde_json::Value = serde_json::from_str(&session).unwrap();

    return v["value"].as_str().unwrap().to_string()
}


