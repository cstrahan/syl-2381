//! # `syl2381` API Documentation

/*
Documentation:
 - https://www.auberins.com/images/Manual/SYL-2381-SSR_manual.pdf
 - https://www.auberins.com/images/Manual/SYL-2381_comm_manual.pdf

Also useful:
 - https://www.ni.com/en-us/shop/seamlessly-connect-to-third-party-devices-and-supervisory-system/the-modbus-protocol-in-depth.html
*/

#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

use core::fmt;

use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

use eh_nb_1_0_alpha as embedded_hal;

mod regs {
    pub const PV: u16 = 0x0164;
    pub const OUT: u16 = 0x0166;
    pub const AL1_STA: u16 = 0x0005;
    pub const CV: u16 = 0x016C;
    pub const AT: u16 = 0x0000;
    pub const SV: u16 = 0x0000;
    pub const AH1: u16 = 0x0002;
    pub const AL1: u16 = 0x0004;
    pub const P: u16 = 0x1000;
    pub const I: u16 = 0x1002;
    pub const D: u16 = 0x1004;
    pub const BB: u16 = 0x1006;
    pub const SOUF: u16 = 0x1008;
    pub const OT: u16 = 0x100A;
    pub const FILT: u16 = 0x100C;
    pub const INTY: u16 = 0x2000;
    pub const OUTY: u16 = 0x2002;
    pub const COTY: u16 = 0x2004;
    pub const HY: u16 = 0x2006;
    pub const PSB: u16 = 0x2008;
    pub const RD: u16 = 0x200A;
    pub const CORF: u16 = 0x200C;
    pub const ID: u16 = 0x200E;
    pub const BAUD: u16 = 0x2010;
}

#[derive(Copy, Clone)]
pub struct Status(u8);

impl Status {
    pub fn alarm1(self) -> bool {
        1 & (self.0 >> 5) == 1
    }

    pub fn anomaly(self) -> bool {
        1 & (self.0 >> 4) == 1
    }

    pub fn setting_mode(self) -> bool {
        1 & (self.0 >> 3) == 1
    }

    pub fn cooling_mode(self) -> bool {
        1 & (self.0 >> 2) == 1
    }

    pub fn manual_mode(self) -> bool {
        1 & (self.0 >> 1) == 1
    }

    pub fn autotune_mode(self) -> bool {
        1 & self.0 == 1
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Status")
            .field("alarm1", &self.alarm1())
            .field("anomaly", &self.anomaly())
            .field("setting_mode", &self.setting_mode())
            .field("cooling_mode", &self.cooling_mode())
            .field("manual_mode", &self.manual_mode())
            .field("autotune_mode", &self.autotune_mode())
            .finish()
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

#[derive(Clone, Copy, fmt::Debug)]
pub enum Filter {
    Disabled,
    Weak,
    Strong,
}

impl TryFrom<f32> for Filter {
    type Error = ();
    fn try_from(value: f32) -> core::result::Result<Self, Self::Error> {
        let val = value as u8;
        let val = match val {
            0 => Filter::Disabled,
            1 => Filter::Weak,
            2 => Filter::Strong,
            _ => return Err(()),
        };

        Ok(val)
    }
}

impl From<Filter> for f32 {
    fn from(value: Filter) -> Self {
        match value {
            Filter::Disabled => 0.0,
            Filter::Weak => 1.0,
            Filter::Strong => 2.0,
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

#[derive(Clone, Copy, fmt::Debug)]
pub enum ControlDirection {
    Heating,
    Cooling,
}

impl TryFrom<f32> for ControlDirection {
    type Error = ();
    fn try_from(value: f32) -> core::result::Result<Self, Self::Error> {
        let val = value as u8;
        let val = match val {
            0 => ControlDirection::Heating,
            1 => ControlDirection::Cooling,
            _ => return Err(()),
        };

        Ok(val)
    }
}

impl From<ControlDirection> for f32 {
    fn from(value: ControlDirection) -> Self {
        match value {
            ControlDirection::Heating => 0.0,
            ControlDirection::Cooling => 1.0,
        }
    }
}

impl fmt::Display for ControlDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

#[derive(Clone, Copy, fmt::Debug)]
pub enum DisplayUnit {
    Celsius,
    Fahrenheit,
}

impl TryFrom<f32> for DisplayUnit {
    type Error = ();
    fn try_from(value: f32) -> core::result::Result<Self, Self::Error> {
        let val = value as u8;
        let val = match val {
            0 => DisplayUnit::Celsius,
            1 => DisplayUnit::Fahrenheit,
            _ => return Err(()),
        };

        Ok(val)
    }
}

impl From<DisplayUnit> for f32 {
    fn from(value: DisplayUnit) -> Self {
        match value {
            DisplayUnit::Celsius => 0.0,
            DisplayUnit::Fahrenheit => 1.0,
        }
    }
}

impl fmt::Display for DisplayUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

#[derive(Clone, Copy, fmt::Debug)]
pub enum BaudRate {
    Baud1200,
    Baud2400,
    Baud4800,
    Baud9600,
}

impl TryFrom<f32> for BaudRate {
    type Error = ();
    fn try_from(value: f32) -> core::result::Result<Self, Self::Error> {
        let val = value as u8;
        let val = match val {
            0 => BaudRate::Baud1200,
            1 => BaudRate::Baud2400,
            2 => BaudRate::Baud4800,
            3 => BaudRate::Baud9600,

            _ => return Err(()),
        };

        Ok(val)
    }
}

impl From<BaudRate> for f32 {
    fn from(value: BaudRate) -> Self {
        match value {
            BaudRate::Baud1200 => 0.0,
            BaudRate::Baud2400 => 1.0,
            BaudRate::Baud4800 => 2.0,
            BaudRate::Baud9600 => 3.0,
        }
    }
}

impl fmt::Display for BaudRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

#[derive(Clone, Copy, fmt::Debug)]
pub enum InputType {
    /// Type T thermocouple.
    T,

    /// Type R thermocouple.
    R,

    /// Type J thermocouple.
    J,

    /// Tungsten Rhenium (WRe 3/25) thermocouple.
    Wre3_25,

    /// Type B thermocouple.
    B,

    /// Type S thermocouple.
    S,

    /// Type K thermocouple.
    K,

    /// Type E thermocouple.
    E,

    /// PT100 RTD at 1 degree of resolution.
    P100,

    /// PT100 RTD at 0.1 degree of resolution.
    P10_0,

    /// Cu50 RTD.
    CU50,
}

impl TryFrom<f32> for InputType {
    type Error = ();
    fn try_from(value: f32) -> core::result::Result<Self, Self::Error> {
        let val = value as u8;
        let val = match val {
            0 => InputType::T,
            1 => InputType::R,
            2 => InputType::J,
            3 => InputType::Wre3_25,
            4 => InputType::B,
            5 => InputType::S,
            6 => InputType::K,
            7 => InputType::E,
            8 => InputType::P100,
            9 => InputType::P10_0,
            10 => InputType::CU50,
            _ => return Err(()),
        };

        Ok(val)
    }
}

impl From<InputType> for f32 {
    fn from(value: InputType) -> Self {
        match value {
            InputType::T => 0.0,
            InputType::R => 1.0,
            InputType::J => 2.0,
            InputType::Wre3_25 => 3.0,
            InputType::B => 4.0,
            InputType::S => 5.0,
            InputType::K => 6.0,
            InputType::E => 7.0,
            InputType::P100 => 8.0,
            InputType::P10_0 => 9.0,
            InputType::CU50 => 10.0,
        }
    }
}

impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

#[derive(Clone, Copy, fmt::Debug)]
pub enum OutputType {
    /// SSR output.
    ///
    /// Not available on SYL-2381-mA-S.
    SSR,

    /// 0-20mA output.
    ///
    /// Not available on SYL-2381-SSR-S.
    #[allow(non_camel_case_types)]
    MA_0_20,

    /// 4-20mA output.
    ///
    /// Not available on SYL-2381-SSR-S.
    #[allow(non_camel_case_types)]
    MA_4_20,
}

impl TryFrom<f32> for OutputType {
    type Error = ();
    fn try_from(value: f32) -> core::result::Result<Self, Self::Error> {
        let val = value as u8;
        let val = match val {
            0 => OutputType::SSR,
            1 => OutputType::MA_0_20,
            2 => OutputType::MA_4_20,
            _ => return Err(()),
        };

        Ok(val)
    }
}

impl From<OutputType> for f32 {
    fn from(value: OutputType) -> Self {
        match value {
            OutputType::SSR => 0.0,
            OutputType::MA_0_20 => 1.0,
            OutputType::MA_4_20 => 2.0,
        }
    }
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

#[derive(Clone, Copy, fmt::Debug)]
pub enum OutputMode {
    /// J1 relay works as absolute alarm output; SSR port as PID control output.
    J1RelayAsAbsoluteAlarmOutputSsrPortAsPidControlOutput,

    /// J1 relay works as derivation alarm output; SSR port as PID control output.
    J1RelayAsDerivationAlarmOutputSsrPortAsPidControlOutput,

    /// J1 relay works as PID control output; SSR port disabled.
    J1RelayAsPidControlOutputSsrPortDisabled,

    /// J1 relay works as on/off control output; SSR port disabled.
    J1RelayAsOnOffControlOutputSsrPortDisabled,

    /// J1 relay works as absolute alarm output; SSR port disabled.
    J1RelayAsAbsoluteAlarmOutputSsrPortDisabled,
}

impl TryFrom<f32> for OutputMode {
    type Error = ();
    fn try_from(value: f32) -> core::result::Result<Self, Self::Error> {
        let val = value as u8;
        let val = match val {
            0 => OutputMode::J1RelayAsAbsoluteAlarmOutputSsrPortAsPidControlOutput,
            1 => OutputMode::J1RelayAsDerivationAlarmOutputSsrPortAsPidControlOutput,
            2 => OutputMode::J1RelayAsPidControlOutputSsrPortDisabled,
            3 => OutputMode::J1RelayAsOnOffControlOutputSsrPortDisabled,
            4 => OutputMode::J1RelayAsAbsoluteAlarmOutputSsrPortDisabled,
            _ => return Err(()),
        };

        Ok(val)
    }
}

impl From<OutputMode> for f32 {
    fn from(value: OutputMode) -> Self {
        match value {
            OutputMode::J1RelayAsAbsoluteAlarmOutputSsrPortAsPidControlOutput => 0.0,
            OutputMode::J1RelayAsDerivationAlarmOutputSsrPortAsPidControlOutput => 1.0,
            OutputMode::J1RelayAsPidControlOutputSsrPortDisabled => 2.0,
            OutputMode::J1RelayAsOnOffControlOutputSsrPortDisabled => 3.0,
            OutputMode::J1RelayAsAbsoluteAlarmOutputSsrPortDisabled => 4.0,
        }
    }
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

pub enum Error<UartError> {
    SerialError(UartError),
    UnexpectedValue(f32),
    ModbusError(rmodbus::ErrorKind),
}

impl<UartError> From<rmodbus::ErrorKind> for Error<UartError> {
    fn from(value: rmodbus::ErrorKind) -> Self {
        Error::ModbusError(value)
    }
}

pub struct Syl2381<UART> {
    unit_id: u8,
    port: UART,
}

impl<UART> Syl2381<UART>
where
    UART: embedded_hal::serial::Read<u8> + embedded_hal::serial::Write<u8>,
{
    pub fn new(unit_id: u8, port: UART) -> Self {
        Syl2381 {
            unit_id: unit_id,
            port: port,
        }
    }

    /// Get the process value (PV).
    pub fn get_pv(&mut self) -> crate::Result<u16, UART> {
        let val = self.get_holding(regs::PV)?;
        Ok(val as u16)
    }

    /// Get the power output percentage (OUT).
    pub fn get_out(&mut self) -> crate::Result<f32, UART> {
        self.get_holding(regs::OUT)
    }

    /// Set the power output percentage (OUT).
    ///
    /// To set the output value, the control flag (CV) must be set.
    pub fn set_out(&mut self, val: f32) -> Result<(), UART> {
        if !(val >= 0.0 && val <= 1.0) {
            return Err(Error::UnexpectedValue(val));
        }
        self.set_holding(regs::OUT, val)
    }

    /// Get J1 status flag (AL1_STA).
    pub fn get_j1_status(&mut self) -> crate::Result<bool, UART> {
        let val = self.get_coils(regs::AL1_STA, 1)?;
        Ok(val & 1 == 1)
    }

    /// Get the control flag for OUT (CV).
    ///
    /// CV controls the function to write/read the parameter OUT:
    /// - When CV is set to 0 (default), host can only read the value for OUT (power output percentage).
    /// - When CV is set to 1, host can read and write OUT.
    ///
    /// It works for both manual mode and PID mode. In PID mode, after you set new
    /// output percentage, the controller itself will not change it (like manual mode).
    ///
    /// To exit, you can either reboot this controller, or set CV back to 0.
    pub fn get_cv(&mut self) -> crate::Result<bool, UART> {
        let val = self.get_holding(regs::CV)?;
        Ok(val == 1.0)
    }

    /// Set the control flag for OUT (CV).
    ///
    /// CV controls the function to write/read the parameter OUT:
    /// - When CV is set to 0 (default), host can only read the value for OUT (power output percentage).
    /// - When CV is set to 1, host can read and write OUT.
    ///
    /// It works for both manual mode and PID mode. In PID mode, after you set new
    /// output percentage, the controller itself will not change it (like manual mode).
    ///
    /// To exit, you can either reboot this controller, or set CV back to 0.
    pub fn set_cv(&mut self, val: bool) -> crate::Result<(), UART> {
        let val = if val { 1.0 } else { 0.0 };
        self.set_holding(regs::CV, val)
    }

    /// Get flag status (AT).
    pub fn get_status(&mut self) -> crate::Result<Status, UART> {
        let val = self.get_coils(regs::AT, 8)?;
        Ok(Status(val))
    }

    /// Get the set value (SV).
    pub fn get_sv(&mut self) -> crate::Result<i16, UART> {
        let val = self.get_holding(regs::SV)?;
        Ok(val as i16)
    }

    /// Set the set value (SV).
    pub fn set_sv(&mut self, val: i16) -> Result<(), UART> {
        if !(val >= -1999 && val <= 9999) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::SV, val)
    }

    /// Get J1 ON temperature (AH1).
    pub fn get_j1_on_temp(&mut self) -> crate::Result<i16, UART> {
        let val = self.get_holding(regs::AH1)?;
        Ok(val as i16)
    }

    /// Set J1 ON temperature (AH1).
    pub fn set_j1_on_temp(&mut self, val: i16) -> Result<(), UART> {
        if !(val >= -1999 && val <= 9999) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::AH1, val)
    }

    /// Get J1 OFF temperature (AL1).
    pub fn get_j1_off_temp(&mut self) -> crate::Result<i16, UART> {
        let val = self.get_holding(regs::AL1)?;
        Ok(val as i16)
    }

    /// Set J1 OFF temperature (AL1).
    pub fn set_j1_off_temp(&mut self, val: i16) -> Result<(), UART> {
        if !(val >= -1999 && val <= 9999) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::AL1, val)
    }

    /// Get proportional constant (P).
    pub fn get_p(&mut self) -> crate::Result<f32, UART> {
        self.get_holding(regs::P)
    }

    /// Get proportional constant (P).
    pub fn set_p(&mut self, val: f32) -> Result<(), UART> {
        if !(val >= -0.1 && val <= 9999.9) {
            return Err(Error::UnexpectedValue(val));
        }
        self.set_holding(regs::P, val)
    }

    /// Get integral time (I).
    pub fn get_i(&mut self) -> crate::Result<u16, UART> {
        let val = self.get_holding(regs::I)?;
        Ok(val as u16)
    }

    /// Set integral time (I).
    pub fn set_i(&mut self, val: u16) -> Result<(), UART> {
        if !(val >= 2 && val <= 1999) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::I, val)
    }

    /// Set derivative time (D).
    pub fn get_d(&mut self) -> crate::Result<u16, UART> {
        let val = self.get_holding(regs::D)?;
        Ok(val as u16)
    }

    /// Set derivative time (D).
    pub fn set_d(&mut self, val: u16) -> Result<(), UART> {
        if !(val <= 999) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::D, val)
    }

    /// Get proportional band range limit (BB).
    pub fn get_bb(&mut self) -> crate::Result<u16, UART> {
        let val = self.get_holding(regs::BB)?;
        Ok(val as u16)
    }

    /// Set proportional band range limit (BB).
    pub fn set_bb(&mut self, val: u16) -> Result<(), UART> {
        if !(val >= 1 && val <= 1999) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::BB, val)
    }

    /// Get the Damp Constant (SouF).
    ///
    /// This constant can help the PID controller further
    /// improve its control quality. It uses the artificial intelligence to dampen the
    /// temperature overshot. When SouF is set to a small value, the system may
    /// overshoot; when SouF is set to a high value, the system will be over-damped.
    pub fn get_souf(&mut self) -> crate::Result<f32, UART> {
        self.get_holding(regs::SOUF)
    }

    /// Set the Damp Constant (SouF).
    ///
    /// This constant can help the PID controller further
    /// improve its control quality. It uses the artificial intelligence to dampen the
    /// temperature overshot. When SouF is set to a small value, the system may
    /// overshoot; when SouF is set to a high value, the system will be over-damped.
    pub fn set_souf(&mut self, val: f32) -> Result<(), UART> {
        if !(val >= 0.0 && val <= 1.0) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        self.set_holding(regs::SOUF, val)
    }

    /// Get control cycle (OT).
    ///
    /// This is a time period setting (unit in seconds) that decides how often
    /// does the controller calculate and change its output.
    pub fn get_control_cycle(&mut self) -> crate::Result<u16, UART> {
        let val = self.get_holding(regs::OT)?;
        Ok(val as u16)
    }

    /// Set control cycle (OT).
    ///
    /// This is a time period setting (unit in seconds) that decides how often
    /// does the controller calculate and change its output.
    pub fn set_control_cycle(&mut self, val: u16) -> Result<(), UART> {
        if !(val >= 1 && val <= 500) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::OT, val)
    }

    /// Get digital filter (FILT).
    ///
    /// NOTE: Stronger filtering increases the stability of
    /// the readout display, but causes more delay in the response to changes in
    /// temperature is a time period setting (unit in seconds) that decides how often
    pub fn get_filter(&mut self) -> crate::Result<Filter, UART> {
        let val = self.get_holding(regs::FILT)?;
        try_from_f32::<_, UART>(val)
    }

    /// Set digital filter (FILT).
    ///
    /// NOTE: Stronger filtering increases the stability of
    /// the readout display, but causes more delay in the response to changes in
    /// temperature is a time period setting (unit in seconds) that decides how often
    pub fn set_filter(&mut self, val: Filter) -> crate::Result<(), UART> {
        let val = val.into();
        self.set_holding(regs::FILT, val)
    }

    /// Get input sensor type (INTY).
    pub fn get_input_sensor_type(&mut self) -> crate::Result<InputType, UART> {
        let val = self.get_holding(regs::INTY)?;
        try_from_f32::<_, UART>(val)
    }

    /// Set input sensor type (INTY).
    pub fn set_input_sensor_type(&mut self, val: InputType) -> crate::Result<(), UART> {
        let val = val.into();
        self.set_holding(regs::INTY, val)
    }

    /// Get output control mode (OUTY).
    pub fn get_output_mode(&mut self) -> crate::Result<OutputMode, UART> {
        let val = self.get_holding(regs::OUTY)?;
        try_from_f32::<_, UART>(val)
    }

    /// Set output control mode (OUTY).
    pub fn set_output_mode(&mut self, val: OutputMode) -> crate::Result<(), UART> {
        let val = val.into();
        self.set_holding(regs::OUTY, val)
    }

    /// Get main output mode (COTY).
    pub fn get_output_type(&mut self) -> crate::Result<OutputType, UART> {
        let val = self.get_holding(regs::COTY)?;
        try_from_f32::<_, UART>(val)
    }

    /// Set main output mode (COTY).
    pub fn set_output_type(&mut self, val: OutputType) -> crate::Result<(), UART> {
        let val = val.into();
        self.set_holding(regs::COTY, val)
    }

    /// Get hysteresis band (Hy).
    pub fn get_hysteresis(&mut self) -> crate::Result<u16, UART> {
        let val = self.get_holding(regs::HY)?;
        Ok(val as u16)
    }

    /// Set hysteresis band (Hy).
    pub fn set_hysteresis(&mut self, val: u16) -> Result<(), UART> {
        if !(val <= 9999) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::HY, val)
    }

    /// Get input offset (PSb).
    pub fn get_input_offset(&mut self) -> crate::Result<i16, UART> {
        let val = self.get_holding(regs::PSB)?;
        Ok(val as i16)
    }

    /// Set input offset (PSb).
    pub fn set_intput_offset(&mut self, val: i16) -> Result<(), UART> {
        if !(val >= -1000 && val <= 1000) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::PSB, val)
    }

    /// Get control function (rd).
    pub fn get_control_direction(&mut self) -> crate::Result<ControlDirection, UART> {
        let val = self.get_holding(regs::RD)?;
        try_from_f32::<_, UART>(val)
    }

    /// Set control function (rd).
    pub fn set_control_direction(&mut self, val: ControlDirection) -> crate::Result<(), UART> {
        let val = val.into();
        self.set_holding(regs::RD, val)
    }

    /// Get display unit (CorF).
    pub fn get_display_unit(&mut self) -> crate::Result<DisplayUnit, UART> {
        let val = self.get_holding(regs::CORF)?;
        try_from_f32::<_, UART>(val)
    }

    /// Set display unit (CorF).
    pub fn set_display_unit(&mut self, val: DisplayUnit) -> crate::Result<(), UART> {
        let val = val.into();
        self.set_holding(regs::CORF, val)
    }

    /// Get unit ID (Id).
    pub fn get_unit_id(&mut self) -> crate::Result<u8, UART> {
        let val = self.get_holding(regs::ID)?;
        Ok(val as u8)
    }

    /// Set unit ID (Id).
    ///
    /// NOTE: This reconfigures the temperature controller to use a different unit ID on the Modbus.
    pub fn set_unit_id(&mut self, val: u8) -> Result<(), UART> {
        if !(val <= 64) {
            return Err(Error::UnexpectedValue(val as f32));
        }
        let val = val as f32;
        self.set_holding(regs::ID, val)
    }

    /// Get baud rate (bAud).
    pub fn get_baud_rate(&mut self) -> crate::Result<BaudRate, UART> {
        let val = self.get_holding(regs::BAUD)?;
        try_from_f32::<_, UART>(val)
    }

    /// Set baud rate (bAud).
    pub fn set_baud_rate(&mut self, val: BaudRate) -> crate::Result<(), UART> {
        let val = val.into();
        self.set_holding(regs::BAUD, val)
    }

    /// ---------------------------

    /// Set holding param.
    ///
    /// All holding params on the SYL-2381 are f32,
    /// encoded as two consecutive values.
    fn set_holding(&mut self, reg: u16, val: f32) -> Result<(), UART> {
        let values = f32_to_values(val);
        let mut mreq = ModbusRequest::new(self.unit_id, ModbusProto::Rtu);

        let mut request: heapless::Vec<u8, 256> = heapless::Vec::new();
        mreq.generate_set_holdings_bulk(reg, &values, &mut request)?;

        self.write_all(&request)?;

        // reuse request buffer
        request.clear();
        let mut response = request;

        // read: addr (byte) + func (byte) + count (byte)
        let _ = response.resize(3, 0);
        self.read_exact(&mut response)?;

        let len = guess_response_frame_len(&response, ModbusProto::Rtu)?;

        let _ = response.resize(len as usize, 0);
        self.read_exact(&mut response[3..])?;

        mreq.parse_ok(&response)?;

        Ok(())
    }

    /// Get holding param.
    ///
    /// All holding params on the SYL-2381 are f32,
    /// encoded as two consecutive values.
    fn get_holding(&mut self, reg: u16) -> Result<f32, UART> {
        let mut mreq = ModbusRequest::new(self.unit_id, ModbusProto::Rtu);

        let mut request: heapless::Vec<u8, 256> = heapless::Vec::new();
        mreq.generate_get_holdings(reg, 2, &mut request)?;

        self.write_all(&request)?;

        // reuse request buffer
        request.clear();
        let mut response = request;

        // read: addr (byte) + func (byte) + count (byte)
        let _ = response.resize(3, 0);
        self.read_exact(&mut response)?;

        let len = guess_response_frame_len(&response, ModbusProto::Rtu)?;

        let _ = response.resize(len as usize, 0);
        self.read_exact(&mut response[3..])?;

        let mut data: heapless::Vec<u16, 2> = heapless::Vec::new();
        mreq.parse_u16(&response, &mut data)?;

        let val = values_to_f32(data[0], data[1]);

        Ok(val)
    }

    /// Get `count` coils.
    ///
    /// We only ever need to read up to 8 consecutive coils from the SYL-2381 (when reading the AT status register),
    /// so this makes the simplifying assumption that we will only ever get 1 byte back.
    fn get_coils(&mut self, reg: u16, count: u8) -> crate::Result<u8, UART> {
        assert!(count <= 8);

        let mut mreq = ModbusRequest::new(self.unit_id, ModbusProto::Rtu);

        let mut request: heapless::Vec<u8, 256> = heapless::Vec::new();
        mreq.generate_get_coils(reg, count as u16, &mut request)?;

        self.write_all(&request)?;

        // reuse request buffer for response
        request.clear();
        let mut response = request;

        // read: addr (byte) + func (byte) + count (byte)
        let _ = response.resize(3, 0);
        self.read_exact(&mut response)?;

        let len = guess_response_frame_len(&response, ModbusProto::Rtu)?;

        let _ = response.resize(len as usize, 0);
        self.read_exact(&mut response[3..])?;
        // println!("response buffer: {:02X?}", response);

        // ensure the response frame was well formed
        mreq.parse_ok(&response)?;

        // As mentioned earlier, only expecting one byte.
        // TODO: new error variant?
        let byte_count = response[2];
        if byte_count != 1 {
            // this should never happen
            return Ok(0);
        }

        // instead of using mreq.parse_bool, which fills a vec of bools,
        // we'll just grab the byte directly.
        // TODO: make this work also work for non-RTU
        let val = response[3];

        Ok(val)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> crate::Result<(), UART> {
        for i in 0..buf.len() {
            let b = nb::block!(self.port.read()).map_err(|err| Error::SerialError(err))?;
            buf[i] = b
        }
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> crate::Result<(), UART> {
        for &b in buf {
            nb::block!(self.port.write(b)).map_err(|err| Error::SerialError(err))?;
        }

        Ok(())
    }
}

pub type Result<T, UART> =
    core::result::Result<T, Error<<UART as embedded_hal::serial::ErrorType>::Error>>;

#[inline(always)]
fn try_from_f32<T, UART>(val: f32) -> crate::Result<T, UART>
where
    T: TryFrom<f32>,
    UART: embedded_hal::serial::ErrorType,
{
    let v = T::try_from(val)
        .map(|v| Ok(v))
        .unwrap_or(Err(Error::UnexpectedValue(val)))?;

    Ok(v)
}

/// Read an f32 from two consecutive holding register values.
#[inline(always)]
fn values_to_f32(d0: u16, d1: u16) -> f32 {
    let [b0, b1] = d0.to_be_bytes();
    let [b2, b3] = d1.to_be_bytes();

    f32::from_be_bytes([b0, b1, b2, b3])
}

/// Splits an f32 into two consecutive holding register values.
#[inline(always)]
fn f32_to_values(val: f32) -> [u16; 2] {
    let [b0, b1, b2, b3] = val.to_be_bytes();
    let d0 = u16::from_be_bytes([b0, b1]);
    let d1 = u16::from_be_bytes([b2, b3]);

    [d0, d1]
}

#[cfg(test)]
mod tests {
    use crate::f32_to_values;
    use crate::values_to_f32;

    #[test]
    fn f32_representation_roundtrips() {
        let f = 10000.0;
        let [d0, d1] = f32_to_values(f);
        let f2 = values_to_f32(d0, d1);
        assert_eq!(f2, f);
    }

    #[test]
    fn f32_read() {
        // 10,000 encoded as two holding register values:
        let d0 = 0x461C;
        let d1 = 0x4000;
        let val = values_to_f32(d0, d1);
        assert_eq!(val, 10_000.0);
    }

    #[test]
    fn f32_write() {
        let val = 10_000.0;
        let vals = f32_to_values(val);
        assert_eq!(vals, [0x461C, 0x4000]);
    }
}
