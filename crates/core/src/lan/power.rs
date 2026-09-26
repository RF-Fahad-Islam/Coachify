// -----------------------------------------------------------------------------
// Win32 Sleep Inhibition
// -----------------------------------------------------------------------------

pub fn set_sleep_inhibition(inhibit: bool) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Power::{
            SetThreadExecutionState, ES_AWAYMODE_REQUIRED, ES_CONTINUOUS, ES_SYSTEM_REQUIRED,
        };
        unsafe {
            if inhibit {
                SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED);
            } else {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = inhibit;
    }
}
