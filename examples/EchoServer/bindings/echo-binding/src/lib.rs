//! The host of the echo server: the one effect its request frame asks for.

use echo_server_host_api::protocol::{tool_calls, Host, HostError};

/// Counts the requests the adapter marks as counted.
#[derive(Default)]
pub struct Native {
    calls: u64,
}

impl Host for Native {
    fn tool_calls(
        &mut self,
        request: tool_calls::Request,
    ) -> Result<tool_calls::Completed, HostError<tool_calls::Failure>> {
        if request.counted {
            self.calls += 1;
        }
        Ok(tool_calls::Completed { calls: self.calls })
    }
}
