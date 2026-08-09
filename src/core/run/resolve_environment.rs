pub struct LaunchEnvironment {
    pub my_os: String,
    pub my_arch: String,
}

pub fn resolve_environment() -> LaunchEnvironment {
    let my_os = match musutils::os::get_os() {
        musutils::os::OS::Windows => "windows",
        musutils::os::OS::Linux => "linux",
        musutils::os::OS::MacOs => "osx",
        _ => "unknown",
    }
    .to_string();

    let my_arch = match musutils::os::get_arch() {
        musutils::os::Arch::X64 => "64",
        musutils::os::Arch::X86 => "32",
        musutils::os::Arch::Arm64 => "arm64",
        musutils::os::Arch::Arm => "arm32",
        _ => "unknown",
    }
    .to_string();

    LaunchEnvironment {
        my_os,
        my_arch,
    }
}
