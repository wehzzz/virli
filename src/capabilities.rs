use caps::{CapSet, Capability};
use std::error::Error;

/// Configures Linux capabilities for the process.
///
/// Clears effective and inheritable capabilities, restricts permitted capabilities
/// to only include `CAP_NET_RAW`, and then raises `CAP_NET_RAW` in the effective,
/// inheritable, and ambient sets.
///
/// This ensures the process has the minimum necessary privileges for networking operations.
pub fn capabilities_configure() -> Result<(), Box<dyn Error>> {
    caps::clear(None, CapSet::Effective)?;
    caps::clear(None, CapSet::Inheritable)?;

    let permitted = caps::read(None, CapSet::Permitted)?;
    for cap in permitted {
        if cap != Capability::CAP_NET_RAW {
            caps::drop(None, CapSet::Permitted, cap)?;
        }
    }

    if !caps::has_cap(None, CapSet::Effective, Capability::CAP_NET_RAW)? {
        caps::raise(None, CapSet::Effective, Capability::CAP_NET_RAW)?;
    }
    if !caps::has_cap(None, CapSet::Inheritable, Capability::CAP_NET_RAW)? {
        caps::raise(None, CapSet::Inheritable, Capability::CAP_NET_RAW)?;
    }
    if !caps::has_cap(None, CapSet::Ambient, Capability::CAP_NET_RAW)? {
        caps::raise(None, CapSet::Ambient, Capability::CAP_NET_RAW)?;
    }

    Ok(())
}
