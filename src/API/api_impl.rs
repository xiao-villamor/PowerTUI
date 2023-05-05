//create mapping between API and rust
//use std::collections::HashMap;

use base64;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;

pub struct VcenterApi{
    pub hostname : String,
}

impl VcenterApi {
    pub(crate) fn new(hostname: String) -> VcenterApi {
        VcenterApi {
            hostname,
        }
    }

    pub(crate) fn clone(&self) -> VcenterApi {
        VcenterApi {
            hostname: self.hostname.clone(),
        }
    }

    pub(crate) fn create_url_for_vcenter(&self) -> String {
        let mut url = String::from("https://");
        url.push_str(self.hostname.as_str());
        url.push_str("/rest/");
        return url;
    }

    pub(crate) fn create_url(&self, host: String) -> String {
        let mut url = String::from("https://");
        url.push_str(self.hostname.as_str());
        url.push_str("/rest/com/vmware/");
        return url;
    }

    pub(crate) fn get_session(&self, username: String, password: String, host: String) -> String {
        let mut url = VcenterApi::create_url(self, host);
        url.push_str("cis/session");

        //create a header map with the authorization header
        let mut headers = HeaderMap::new();
        let mut auth = String::from("Basic ");
        let mut auth_string = String::from(username);
        auth_string.push_str(":");
        auth_string.push_str(&password);
        auth.push_str(&base64::encode(&auth_string));
        headers.insert("Authorization", auth.parse().unwrap());

        let client = Client::builder();
        let response = client
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .post(&url)
            .headers(headers)
            .send();


        //get the response
        let response = response.unwrap();

        //get the response body
        let body = response.text().unwrap();

        return body;
    }

    pub(crate) fn get_all_vms(&self, credentials: String) -> String {
        let mut url = VcenterApi::create_url_for_vcenter(self);
        url.push_str("vcenter/vm");

        //create a header map with the authorization header
        let mut headers = HeaderMap::new();
        let auth = String::from(&credentials);
        headers.insert("vmware-api-session-id", auth.parse().unwrap());


        let client = Client::builder();
        let response = client
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(&url)
            .headers(headers)
            .send();

        let response = response.unwrap();

        //get the response body
        let body = response.text().unwrap();

        //convert to json
        //convert the json to object


        return body;
    }

    pub(crate) fn get_vms_from_host(&self, credentials: String, host: String) -> String {
        let mut url = VcenterApi::create_url_for_vcenter(self);
        url.push_str("vcenter/vm");
        url.push_str("?filter.hosts=");
        url.push_str(&host);

        //create a header map with the authorization header
        let mut headers = HeaderMap::new();
        let auth = String::from(&credentials);
        headers.insert("vmware-api-session-id", auth.parse().unwrap());


        let client = Client::builder();
        let response = client
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(&url)
            .headers(headers)
            .send();

        let response = response.unwrap();

        //get the response body
        let body = response.text().unwrap();

        //convert to json

        //convert the json to object
        return body;
    }

    pub(crate) fn get_all_hosts(&self, credentials: String) -> String {
        let mut url = VcenterApi::create_url_for_vcenter(self);
        url.push_str("vcenter/host");

        //create a header map with the authorization header
        let mut headers = HeaderMap::new();
        let auth = String::from(&credentials);
        headers.insert("vmware-api-session-id", auth.parse().unwrap());

        let client = Client::builder();
        let response = client
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(&url)
            .headers(headers)
            .send();

        let response = response.unwrap();

        //get the response body
        let body = response.text().unwrap();

        //convert the json to object
        return body;
    }

    pub(crate) fn get_host(&self, credentials: String, host: String) -> String {
        let mut url = VcenterApi::create_url_for_vcenter(self);
        url.push_str("vcenter/host/");
        url.push_str("?filter.names=");
        url.push_str(&host);

        println!("{}", url);

        //create a header map with the authorization header
        let mut headers = HeaderMap::new();
        let auth = String::from(&credentials);
        headers.insert("vmware-api-session-id", auth.parse().unwrap());


        let client = Client::builder();
        let response = client
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(&url)
            .headers(headers)
            .send();

        let response = response.unwrap();

        //get the response body
        let body = response.text().unwrap();

        //convert the json to object
        return body;
    }

    pub(crate) fn shutdown_vm(&self, credentials: String, vm: String) -> StatusCode {
        let mut url = VcenterApi::create_url_for_vcenter(self);
        url.push_str("vcenter/vm/");
        url.push_str(&vm);
        url.push_str("/power/stop");

        //create a header map with the authorization header
        let mut headers = HeaderMap::new();
        let auth = String::from(&credentials);
        headers.insert("vmware-api-session-id", auth.parse().unwrap());

        let client = Client::builder();
        let response = client
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .post(&url)
            .headers(headers)
            .send();

        let response = response.unwrap();
        //get status code
        let status_code = response.status();

        return status_code;
    }

    pub(crate) fn start_vm(&self, credentials: String, vm: String) -> StatusCode {
        let mut url = VcenterApi::create_url_for_vcenter(self);
        url.push_str("vcenter/vm/");
        url.push_str(&vm);
        url.push_str("/power/start");

        //create a header map with the authorization header
        let mut headers = HeaderMap::new();
        let auth = String::from(&credentials);
        headers.insert("vmware-api-session-id", auth.parse().unwrap());

        let client = Client::builder();
        let response = client
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .post(&url)
            .headers(headers)
            .send();

        let response = response.unwrap();
        //get status code
        let status_code = response.status();

        return status_code;
    }

    pub(crate) fn reboot_vm(&self, credentials : String, vm : String) -> StatusCode {
        let mut url = VcenterApi::create_url_for_vcenter(self);
        url.push_str("vcenter/vm/");
        url.push_str(&vm);
        url.push_str("/power/reset");

        //create a header map with the authorization header
        let mut headers = HeaderMap::new();
        let auth = String::from(&credentials);
        headers.insert("vmware-api-session-id", auth.parse().unwrap());

        let client = Client::builder();
        let response = client
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .post(&url)
            .headers(headers)
            .send();

        let response = response.unwrap();
        //get status code
        let status_code = response.status();

        return status_code;
    }
}





