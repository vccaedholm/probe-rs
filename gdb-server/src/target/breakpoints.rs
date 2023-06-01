use super::{GdbErrorExt, RuntimeTarget};

use probe_rs::architecture::arm::{
    component::{add_watchpoint, remove_watchpoint},
    DpAddress,
};

use gdbstub::target::ext::breakpoints::{
    Breakpoints, HwBreakpoint, HwBreakpointOps, HwWatchpoint, HwWatchpointOps, SwBreakpointOps,
    WatchKind,
};
use probe_rs::architecture::arm::component::WatchKind as ProbeRsWatchKind;

impl Breakpoints for RuntimeTarget<'_> {
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        None
    }

    fn support_hw_breakpoint(&mut self) -> Option<HwBreakpointOps<'_, Self>> {
        Some(self)
    }

    fn support_hw_watchpoint(&mut self) -> Option<HwWatchpointOps<'_, Self>> {
        Some(self)
    }
}

impl HwBreakpoint for RuntimeTarget<'_> {
    fn add_hw_breakpoint(
        &mut self,
        addr: u64,
        _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        let mut session = self.session.lock().unwrap();

        for core_id in &self.cores {
            let mut core = session.core(*core_id).into_target_result()?;

            core.set_hw_breakpoint(addr).into_target_result()?;
        }

        Ok(true)
    }

    fn remove_hw_breakpoint(
        &mut self,
        addr: u64,
        _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        let mut session = self.session.lock().unwrap();

        for core_id in &self.cores {
            let mut core = session.core(*core_id).into_target_result()?;

            core.clear_hw_breakpoint(addr).into_target_result()?;
        }

        Ok(true)
    }
}

impl HwWatchpoint for RuntimeTarget<'_> {
    // Required methods
    fn add_hw_watchpoint(
        &mut self,
        addr: u64,
        len: u64,
        kind: WatchKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        let mut session = self.session.lock().unwrap();
        let components = session.get_arm_components(DpAddress::Default).unwrap();
        let interface = session.get_arm_interface().unwrap();

        let probe_rs_kind = match kind {
            WatchKind::Read => ProbeRsWatchKind::Read,
            WatchKind::Write => ProbeRsWatchKind::Write,
            WatchKind::ReadWrite => ProbeRsWatchKind::ReadWrite,
        };

        add_watchpoint(
            interface,
            &components,
            0, // FIXME: hardcoded first unit
            addr as u32,
            len as usize,
            probe_rs_kind,
        )
        .unwrap();
        Ok(true)
    }

    fn remove_hw_watchpoint(
        &mut self,
        _addr: u64,
        _len: u64,
        _kind: WatchKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        let mut session = self.session.lock().unwrap();
        let components = session.get_arm_components(DpAddress::Default).unwrap();
        let interface = session.get_arm_interface().unwrap();

        remove_watchpoint(
            interface,
            &components,
            0, // FIXME: hardcoded first unit
        )
        .unwrap();
        Ok(true)
    }
}
