use std::env;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::null;
use winapi::um::{winnt, winsvc};
use anyhow::{anyhow, Result};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winnt::DELETE;
use winapi::um::winsvc::{SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS, SERVICE_START, SERVICE_STOP};

#[test]
pub fn test_api() {
    println!("{:?}", installDokanDriver_API());
}

pub fn installDokanDriver_API() -> Result<bool> {
    let serviceName = OsStr::new("dokan1").encode_wide().chain(once(0)).collect::<Vec<u16>>();
    let servicePath = Path::new(&env::var("windir")?).join(r"System32\drivers\dokan1.sys");
    let servicePath = servicePath.as_os_str().encode_wide().chain(once(0)).collect::<Vec<u16>>();

    unsafe {
        let controlHandle = winsvc::OpenSCManagerW(null(), null(), winsvc::SC_MANAGER_CREATE_SERVICE);
        if controlHandle.is_null() {
            return Err(anyhow!("Failed to open Service Control"));
        }

        let serviceHandle = winsvc::CreateServiceW(controlHandle, serviceName.as_ptr(), serviceName.as_ptr(), 0, 2,
                                                   winnt::SERVICE_AUTO_START,
                                                   winnt::SERVICE_ERROR_IGNORE,
                                                   servicePath.as_ptr(),
                                                   null(), &mut 0_u32, null(), null(), null());
        if serviceHandle.is_null() {
            let lastError = GetLastError();
            println!("{}", lastError);

            winsvc::CloseServiceHandle(controlHandle);
            return if lastError == winapi::shared::winerror::ERROR_SERVICE_EXISTS {
                Err(anyhow!("Service is already installed"))
            } else {
                Err(anyhow!("Failed to install service"))
            };
        }
        println!("{:?}", serviceHandle);
        println!("{:?}", serviceHandle.is_null());
        winsvc::CloseServiceHandle(serviceHandle);
        winsvc::CloseServiceHandle(controlHandle);

        let controlHandle = winsvc::OpenSCManagerW(null(), null(), winsvc::SC_MANAGER_CONNECT);
        if controlHandle.is_null() {
            return Err(anyhow!("Failed to open Service Control"));
        }
        let serviceHandle = winapi::um::winsvc::OpenServiceW(controlHandle, serviceName.as_ptr(), SERVICE_START | SERVICE_STOP | SERVICE_QUERY_STATUS | DELETE);
        if serviceHandle.is_null() {
            winsvc::CloseServiceHandle(controlHandle);
            return Err(anyhow!("Failed to open Service"));
        }
    }
    Ok(false)
}
