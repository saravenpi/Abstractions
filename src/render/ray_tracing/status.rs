//! The one status line of the graphics settings page.

use super::RayTracingBoot;
use super::boot::Start;
use super::gpu::Support;

/// Returns the line shown under the graphics settings.
pub fn status(boot: RayTracingBoot, active: bool) -> &'static str {
    match (boot.guard, boot.support) {
        (Start::Recovered, _) => "Ray tracing is not available: disabled after a failed start",
        (Start::Unwritable, _) => {
            "Ray tracing is not available: the start guard could not be written"
        }
        (Start::Disabled, _) => {
            "Ray tracing is not available: turned off by ABSTRACTIONS_RAY_TRACING"
        }
        (_, Support::NoHardware) => {
            "Ray tracing is not available: this GPU has no hardware ray tracing"
        }
        (_, Support::TooLimited) => "Ray tracing is not available: this GPU is too limited for it",
        (_, Support::Unchecked) => "Ray tracing is not available: the GPU was not checked",
        (_, Support::Ready) if active => "Ray tracing is on",
        (_, Support::Ready) => "Ray tracing is off",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The line names the first blocker, then the running state.
    #[test]
    fn status_follows_the_blocker() {
        let ready = RayTracingBoot {
            guard: Start::Armed,
            support: Support::Ready,
        };
        let no_gpu = RayTracingBoot {
            support: Support::NoHardware,
            ..ready
        };
        let failed = RayTracingBoot {
            guard: Start::Recovered,
            ..ready
        };
        assert_eq!(status(ready, true), "Ray tracing is on");
        assert_eq!(status(ready, false), "Ray tracing is off");
        assert!(status(no_gpu, false).contains("no hardware ray tracing"));
        assert!(status(failed, false).contains("failed start"));
        let switched = RayTracingBoot {
            guard: Start::Disabled,
            ..ready
        };
        assert!(status(switched, false).contains("ABSTRACTIONS_RAY_TRACING"));
    }
}
