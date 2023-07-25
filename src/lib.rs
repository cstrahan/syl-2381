/*
Documentation:
 - https://www.auberins.com/images/Manual/SYL-2381-SSR_manual.pdf
 - https://www.auberins.com/images/Manual/SYL-2381_comm_manual.pdf

Also useful:
 - https://www.ni.com/en-us/shop/seamlessly-connect-to-third-party-devices-and-supervisory-system/the-modbus-protocol-in-depth.html
*/
// use serial::unix::TTYPort;
use serialport::{SerialPort, TTYPort};
use std::io::{Read, Write};

use std::fmt;

use rmodbus::{client::ModbusRequest, guess_response_frame_len, ModbusProto};

use eh_nb_1_0_alpha as embedded_hal;

mod r {
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

// D7 = 0: Reserved parameter.
// D6 = 0: Reserved parameter.
// D5 = 1: Alarm 1 is activated; D5 = 0: Alarm 1 is deactivated.
// D4 = 1: Controller is in anomaly status. For example, sensor is not connected and display shows EEEE. D4 = 0: Normal status.
// D3 = 1: Controller is in setting mode of static parameters (section 3.2); D3 = 0: Normal status.
// D2 = 1: Cooling mode; D2 = 0: Heating mode
// D1 = 1: Controller is in manual mode; D1 = 0: Normal status.
// D0 = 1: Controller is in the process of auto-tune; D0 = 0: Normal status.
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

#[derive(Clone, Copy, fmt::Debug)]
pub enum Filter {
    Disabled,
    Weak,
    Strong,
}

impl TryFrom<f32> for Filter {
    type Error = ();
    fn try_from(value: f32) -> Result<Self, Self::Error> {
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
    fn try_from(value: f32) -> Result<Self, Self::Error> {
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
    fn try_from(value: f32) -> Result<Self, Self::Error> {
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
    fn try_from(value: f32) -> Result<Self, Self::Error> {
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
    fn try_from(value: f32) -> Result<Self, Self::Error> {
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
    fn try_from(value: f32) -> Result<Self, Self::Error> {
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
    fn try_from(value: f32) -> Result<Self, Self::Error> {
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

pub struct Syl2381<S> {
    unit_id: u8,
    port: S,
}

impl<S> Syl2381<S>
where
    S: embedded_hal::serial::Read<u8> + embedded_hal::serial::Write<u8>,
{
    pub fn new(unit_id: u8, port: S) -> Self {
        Syl2381 {
            unit_id: unit_id,
            port: port,
        }
    }

    /// Get the process value (PV).
    pub fn get_pv(&mut self) -> f32 {
        self.get_holding(r::PV)
    }

    /// Get the power output percentage (OUT).
    pub fn get_out(&mut self) -> f32 {
        self.get_holding(r::OUT)
    }

    /// Set the power output percentage (OUT).
    ///
    /// To set the output value, the control flag (CV) must be set.
    pub fn set_out(&mut self, val: f32) -> f32 {
        assert!(val >= 0.0 && val <= 1.0);
        self.get_holding(r::OUT)
    }

    /// Get J1 status flag (AL1_STA).
    pub fn get_al1_sta(&mut self) -> bool {
        self.get_coils(r::AL1_STA, 1) & 1 == 1
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
    pub fn get_cv(&mut self) -> bool {
        self.get_holding(r::CV) == 1.0
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
    pub fn set_cv(&mut self, val: bool) {
        let val = if val { 1.0 } else { 0.0 };
        self.set_holding(r::CV, val)
    }

    /// Get flag status (AT).
    pub fn get_at(&mut self) -> Status {
        Status(self.get_coils(r::AT, 8))
    }

    /// Get the set value (SV).
    pub fn get_sv(&mut self) -> f32 {
        self.get_holding(r::SV)
    }

    /// Set the set value (SV).
    pub fn set_sv(&mut self, val: f32) {
        assert!(val > -1999.0 && val < 9999.0);
        self.set_holding(r::SV, val)
    }

    /// Get J1 ON temperature (AH1).
    pub fn get_ah1(&mut self) -> f32 {
        self.get_holding(r::AH1)
    }

    /// Set J1 ON temperature (AH1).
    pub fn set_ah1(&mut self, val: f32) {
        assert!(val > -1999.0 && val < 9999.0);
        self.set_holding(r::AH1, val)
    }

    /// Get J1 OFF temperature (AL1).
    pub fn get_al1(&mut self) -> f32 {
        self.get_holding(r::AL1)
    }

    /// Set J1 OFF temperature (AL1).
    pub fn set_al1(&mut self, val: f32) {
        assert!(val >= -1999.0 && val <= 9999.0);
        self.set_holding(r::AL1, val)
    }

    /// Get proportional constant (P).
    pub fn get_p(&mut self) -> f32 {
        self.get_holding(r::P)
    }

    /// Get proportional constant (P).
    pub fn set_p(&mut self, val: f32) {
        assert!(val >= 0.1 && val <= 9999.9);
        self.set_holding(r::P, val)
    }

    /// Get integral time (I).
    pub fn get_i(&mut self) -> f32 {
        self.get_holding(r::I)
    }

    /// Set integral time (I).
    pub fn set_i(&mut self, val: f32) {
        assert!(val >= -0.0 && val <= 9999.0);
        self.set_holding(r::I, val)
    }

    /// Set derivative time (D).
    pub fn get_d(&mut self) -> f32 {
        self.get_holding(r::D)
    }

    /// Set derivative time (D).
    pub fn set_d(&mut self, val: f32) {
        assert!(val >= 0.0 && val <= 999.0);
        self.set_holding(r::D, val)
    }

    /// Get proportional band range limit (BB).
    pub fn get_bb(&mut self) -> f32 {
        self.get_holding(r::BB)
    }

    /// Set proportional band range limit (BB).
    pub fn set_bb(&mut self, val: f32) {
        assert!(val >= 1.0 && val <= 1999.0);
        self.set_holding(r::BB, val)
    }

    /// Get the Damp Constant (SouF).
    ///
    /// This constant can help the PID controller further
    /// improve its control quality. It uses the artificial intelligence to dampen the
    /// temperature overshot. When SouF is set to a small value, the system may
    /// overshoot; when SouF is set to a high value, the system will be over-damped.
    pub fn get_souf(&mut self) -> f32 {
        self.get_holding(r::SOUF)
    }

    /// Set the Damp Constant (SouF).
    ///
    /// This constant can help the PID controller further
    /// improve its control quality. It uses the artificial intelligence to dampen the
    /// temperature overshot. When SouF is set to a small value, the system may
    /// overshoot; when SouF is set to a high value, the system will be over-damped.
    pub fn set_souf(&mut self, val: f32) {
        assert!(val >= 0.0 && val <= 1.0);
        self.set_holding(r::SOUF, val)
    }

    /// Get control cycle (OT).
    ///
    /// This is a time period setting (unit in seconds) that decides how often
    /// does the controller calculate and change its output.
    pub fn get_ot(&mut self) -> f32 {
        self.get_holding(r::OT)
    }

    /// Set control cycle (OT).
    ///
    /// This is a time period setting (unit in seconds) that decides how often
    /// does the controller calculate and change its output.
    pub fn set_ot(&mut self, val: f32) {
        assert!(val >= 1.0 && val <= 500.0);
        self.set_holding(r::SOUF, val)
    }

    /// Get digital filter (FILT).
    ///
    /// NOTE: Stronger filtering increases the stability of
    /// the readout display, but causes more delay in the response to changes in
    /// temperature is a time period setting (unit in seconds) that decides how often
    pub fn get_filt(&mut self) -> Filter {
        let val = self.get_holding(r::FILT);
        val.try_into().unwrap()
    }

    /// Set digital filter (FILT).
    ///
    /// NOTE: Stronger filtering increases the stability of
    /// the readout display, but causes more delay in the response to changes in
    /// temperature is a time period setting (unit in seconds) that decides how often
    pub fn set_filt(&mut self, val: Filter) {
        let val = val.into();
        self.set_holding(r::FILT, val)
    }

    /// Get input sensor type (INTY).
    pub fn get_inty(&mut self) -> InputType {
        let val = self.get_holding(r::INTY);
        val.try_into().unwrap()
    }

    /// Set input sensor type (INTY).
    pub fn set_inty(&mut self, val: InputType) {
        let val = val.into();
        self.set_holding(r::INTY, val)
    }

    /// Get output control mode (OUTY).
    pub fn get_outy(&mut self) -> OutputMode {
        let val = self.get_holding(r::OUTY);
        val.try_into().unwrap()
    }

    /// Set output control mode (OUTY).
    pub fn set_outy(&mut self, val: OutputMode) {
        let val = val.into();
        self.set_holding(r::OUTY, val)
    }

    /// Get main output mode (OUTY).
    pub fn get_coty(&mut self) -> OutputType {
        let val = self.get_holding(r::COTY);
        val.try_into().unwrap()
    }

    /// Set main output mode (OUTY).
    pub fn set_coty(&mut self, val: OutputType) {
        let val = val.into();
        self.set_holding(r::COTY, val)
    }

    /// Get hysteresis band (Hy).
    pub fn get_hy(&mut self) -> f32 {
        self.get_holding(r::HY)
    }

    /// Set hysteresis band (Hy).
    pub fn set_hy(&mut self, val: f32) {
        assert!(val >= 0.0 && val <= 9999.0);
        self.set_holding(r::HY, val)
    }

    /// Get input offset (PSb).
    pub fn get_psb(&mut self) -> f32 {
        self.get_holding(r::PSB)
    }

    /// Set input offset (PSb).
    pub fn set_psb(&mut self, val: f32) {
        assert!(val >= -1000.0 && val <= 1000.0);
        self.set_holding(r::HY, val)
    }

    /// Get control function (rd).
    pub fn get_rd(&mut self) -> ControlDirection {
        let val = self.get_holding(r::RD);
        val.try_into().unwrap()
    }

    /// Set control function (rd).
    pub fn set_rd(&mut self, val: ControlDirection) {
        let val = val.into();
        self.set_holding(r::RD, val)
    }

    /// Get display unit (CorF).
    pub fn get_corf(&mut self) -> DisplayUnit {
        let val = self.get_holding(r::CORF);
        val.try_into().unwrap()
    }

    /// Set display unit (CorF).
    pub fn set_corf(&mut self, val: DisplayUnit) {
        let val = val.into();
        self.set_holding(r::CORF, val)
    }

    /// Get unit ID (Id).
    pub fn get_id(&mut self) -> f32 {
        self.get_holding(r::ID)
    }

    /// Set unit ID (Id).
    pub fn set_id(&mut self, val: f32) {
        assert!(val >= 0.0 && val <= 64.0);
        self.set_holding(r::ID, val)
    }

    /// Get baud rate (bAud).
    pub fn get_baud(&mut self) -> BaudRate {
        let val = self.get_holding(r::BAUD);
        val.try_into().unwrap()
    }

    /// Set baud rate (bAud).
    pub fn set_baud(&mut self, val: BaudRate) {
        let val = val.into();
        self.set_holding(r::BAUD, val)
    }

    /// ---------------------------

    /// All holding values on the SYL-2381 are f32,
    /// encoded as two consecutive values.
    fn set_holding(&mut self, reg: u16, val: f32) {
        let values = values_to_f32(val);
        let mut mreq = ModbusRequest::new(self.unit_id, ModbusProto::Rtu);

        let mut request: heapless::Vec<u8, 256> = heapless::Vec::new();
        mreq.generate_set_holdings_bulk(reg, &values, &mut request)
            .expect("modbus gen");

        self.write_all(&request);

        let mut buf = [0u8; 3];

        self.read_all(&mut buf);

        let mut response = Vec::new();
        response.extend_from_slice(&buf);
        let len = guess_response_frame_len(&buf, ModbusProto::Rtu).expect("guess len");

        // for RTU: addr (byte) + func (byte) + count (byte) + payload (count bytes) + crc (2 bytes)
        // so we subtract 3 bytes from the full frame len to get the remaining length.
        if len > 3 {
            let mut rest = vec![0u8; (len - 3) as usize];
            self.read_all(&mut rest);
            response.extend(rest);
        }

        mreq.parse_ok(&response).expect("writing holding");
    }

    fn get_holding(&mut self, reg: u16) -> f32 {
        let mut mreq = ModbusRequest::new(self.unit_id, ModbusProto::Rtu);

        // request is 8 bytes, response is 9 bytes.
        // use 9 byte capacity so we can reuse buffer for response.
        let mut request: heapless::Vec<u8, 9> = heapless::Vec::new();
        mreq.generate_get_holdings(reg, 2, &mut request)
            .expect("modbus gen");

        self.write_all(&request);

        let mut buf = [0u8; 3];
        self.read_all(&mut buf);

        // reuse request buffer
        request.clear();
        let mut response = request;

        response.extend_from_slice(&buf);
        let len = guess_response_frame_len(&buf, ModbusProto::Rtu).expect("guess len");

        // for RTU: addr (byte) + func (byte) + count (byte) + payload (count bytes) + crc (2 bytes)
        // so we subtract 3 bytes from the full frame len to get the remaining length.
        if len > 3 {
            let mut rest = vec![0u8; (len - 3) as usize];
            self.read_all(&mut rest);
            response.extend(rest);
        }

        let mut data: heapless::Vec<u16, 2> = heapless::Vec::new();
        mreq.parse_u16(&response, &mut data).unwrap();

        let val = f32_to_values(data[0], data[1]);

        val
    }

    /// Get `count` coils.
    ///
    /// We only ever need to read up to 8 consecutive coils from the SYL-2381 (when reading the AT status register),
    /// so this makes the simplifying assumption that we will only ever get 1 byte back.
    fn get_coils(&mut self, reg: u16, count: u8) -> u8 {
        assert!(count <= 8);

        let mut mreq = ModbusRequest::new(self.unit_id, ModbusProto::Rtu);

        // request is 8 bytes, response is 6 bytes.
        // use 8 byte capacity so we can reuse buffer for response.
        let mut request: heapless::Vec<u8, 9> = heapless::Vec::new();
        mreq.generate_get_coils(reg, count as u16, &mut request)
            .expect("modbus gen");

        self.write_all(&request);

        let mut buf = [0u8; 3];
        self.read_all(&mut buf);

        // TODO: don't hardcode this around RTU
        let byte_count = buf[2];
        // As mentioned earlier, only expecting one byte.
        assert_eq!(byte_count, 1);

        // reuse request buffer for response
        request.clear();
        let mut response = request;

        response.extend_from_slice(&buf);
        let len = guess_response_frame_len(&buf, ModbusProto::Rtu).expect("guess len");

        // for RTU: addr (byte) + func (byte) + count (byte) + payload (count bytes) + crc (2 bytes)
        // so we subtract 3 bytes from the full frame len to get the remaining length.
        if len > 3 {
            let mut rest = vec![0u8; (len - 3) as usize];
            self.read_all(&mut rest);
            response.extend(rest);
        }

        // println!("response buffer: {:02X?}", response);

        // ensure the response frame was well formed
        mreq.parse_ok(&response).expect("response parse");
        // instead of using mreq.parse_bool, which fills a vec of bools,
        // we'll just grab the byte directly.
        // TODO: make this work also work for non-RTU
        let val = response[3];
        val
    }

    fn read_all(
        &mut self,
        buf: &mut [u8],
    ) -> Result<(), <S as embedded_hal::serial::ErrorType>::Error> {
        for i in 0..buf.len() {
            let b = nb::block!(self.port.read())?;
            buf[i] = b
        }
        Ok(())
    }

    fn write_all(
        &mut self,
        buf: &[u8],
    ) -> Result<(), <S as embedded_hal::serial::ErrorType>::Error> {
        for &b in buf {
            nb::block!(self.port.write(b))?;
        }

        Ok(())
    }
}

/// Read an f32 from two consecutive holding register values.
#[inline(always)]
fn f32_to_values(d0: u16, d1: u16) -> f32 {
    let w0 = d0.to_be_bytes();
    let w1 = d1.to_be_bytes();
    let fbits = (w0[0] as u32) << 24 | (w0[1] as u32) << 16 | (w1[0] as u32) << 8 | (w1[1] as u32);
    let val = f32::from_bits(fbits);
    val
}

/// Splits an f32 into two consecutive holding register values.
fn values_to_f32(val: f32) -> [u16; 2] {
    let bytes = val.to_be_bytes();
    let d0 = (bytes[0] as u16) << 8 | bytes[1] as u16;
    let d1 = (bytes[2] as u16) << 8 | bytes[3] as u16;

    [d0, d1]
}

#[cfg(test)]
mod tests {
    use crate::f32_to_values;
    use crate::values_to_f32;

    #[test]
    fn f32_representation_roundtrips() {
        let f = 10000.0;
        let [d0, d1] = values_to_f32(f);
        let f2 = f32_to_values(d0, d1);
        assert_eq!(f2, f);
    }

    #[test]
    fn f32_read() {
        // 10,000 encoded as two holding register values:
        let d0 = 0x461C;
        let d1 = 0x4000;
        let val = f32_to_values(d0, d1);
        assert_eq!(val, 10_000.0);
    }

    #[test]
    fn f32_write() {
        let val = 10_000.0;
        let vals = values_to_f32(val);
        assert_eq!(vals, [0x461C, 0x4000]);
    }
}
