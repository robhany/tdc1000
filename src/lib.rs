#![no_std]

extern crate embedded_hal as hal;

use hal::{
    blocking::spi::{Transfer, Write},
    digital::v2::OutputPin,
};

const SPI_WRITE_BIT: u8 = 0x40;

#[derive(Debug)]
pub enum Error<CsE, SpiE> {
    ChipSelectError(CsE),
    SpiError(SpiE),
}

#[repr(u8)]
enum ConfigAddresses {
    Config0,
    Config1,
    Config2,
    Config3,
    Config4,
    Tof1,
    Tof0,
    ErrFlag,
    TimeOut,
    ClockRate,
}

const FREQUENCY_DIVIDER_BIT_OFFSET: u8 = 5;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TxFrequencyDivider {
    DivideBy2,
    DivideBy4,
    DivideBy8,
    DivideBy16,
    DivideBy32,
    DivideBy64,
    DivideBy128,
    DivideBy256,
}
impl Default for TxFrequencyDivider {
    fn default() -> Self {
        TxFrequencyDivider::DivideBy8
    }
}

#[derive(Copy, Clone)]
pub struct TxPulses(u8);
impl TxPulses {
    pub const LOW: u8 = 0;
    pub const HIGH: u8 = 31;
    pub fn new(pulses: u8) -> Self {
        TxPulses(trim_value_u8(pulses, Self::HIGH, Self::LOW))
    }
    pub fn get_value(&self) -> u8 {
        self.0
    }
}

impl Default for TxPulses {
    fn default() -> Self {
        TxPulses::new(5)
    }
}

const MEASUREMENT_CYCLES_BIT_OFFSET: u8 = 3;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum MeasurementCycles {
    MeasurementCycles1,
    MeasurementCycles2,
    MeasurementCycles4,
    MeasurementCycles8,
    MeasurementCycles16,
    MeasurementCycles32,
    MeasurementCycles64,
    MeasurementCycles128,
}
impl Default for MeasurementCycles {
    fn default() -> Self {
        MeasurementCycles::MeasurementCycles1
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ReceiveEventsCnt {
    DoNotCountStopEvents,
    StopEvents1,
    StopEvents2,
    StopEvents3,
    StopEvents4,
    StopEvents5,
    StopEvents6,
    StopEvents7,
}

impl Default for ReceiveEventsCnt {
    fn default() -> Self {
        ReceiveEventsCnt::DoNotCountStopEvents
    }
}

const VOLTAGE_REFERENCE_BIT_OFFSET: u8 = 7;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum VoltageReference {
    Internal,
    External,
}
impl Default for VoltageReference {
    fn default() -> Self {
        VoltageReference::Internal
    }
}

const MEASUREMENT_MODE_BIT_OFFSET: u8 = 6;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum MeasurementMode {
    TimeOfFlight,
    Temperature,
}
impl Default for MeasurementMode {
    fn default() -> Self {
        MeasurementMode::TimeOfFlight
    }
}

const DAMPING_MODE_BIT_OFFSET: u8 = 5;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum DampingMode {
    DisableDamping,
    EnableDamping,
}
impl Default for DampingMode {
    fn default() -> Self {
        DampingMode::DisableDamping
    }
}

const CHANNEL_SWAP_BIT_OFFSET: u8 = 4;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ChannelSwap {
    DisableSwap,
    EnableSwap,
}
impl Default for ChannelSwap {
    fn default() -> Self {
        ChannelSwap::DisableSwap
    }
}

const EXTERNAL_CHANNEL_SELECT_BIT_OFFSET: u8 = 3;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ExternalChannelSelect {
    DisableExternalChannelSelect,
    EnableExternalChannelSelect,
}
impl Default for ExternalChannelSelect {
    fn default() -> Self {
        ExternalChannelSelect::DisableExternalChannelSelect
    }
}

const CHANNEL_SELECT_BIT_OFFSET: u8 = 2;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ChannelSelect {
    Channel1,
    Channel2,
}
impl Default for ChannelSelect {
    fn default() -> Self {
        ChannelSelect::Channel1
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TOFMeasurementMode {
    Mode0,
    Mode1,
    Mode2,
}
impl Default for TOFMeasurementMode {
    fn default() -> Self {
        TOFMeasurementMode::Mode0
    }
}

const TEMP_MODE_BIT_OFFSET: u8 = 6;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TempMode {
    MeasureRefRtd1Rtd2,
    MeasureRefRtd1,
}
impl Default for TempMode {
    fn default() -> Self {
        TempMode::MeasureRefRtd1Rtd2
    }
}

const TEMP_RTD_SELECT_BIT_OFFSET: u8 = 5;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TempRtdSelect {
    PT1000,
    PT500,
}
impl Default for TempRtdSelect {
    fn default() -> Self {
        TempRtdSelect::PT1000
    }
}

const TEMP_CLK_DIV_BIT_OFFSET: u8 = 4;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TempClockDivider {
    DivideBy8,
    UseTxFreqDivider,
}
impl Default for TempClockDivider {
    fn default() -> Self {
        TempClockDivider::DivideBy8
    }
}

const BLANKING_BIT_OFFSET: u8 = 3;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PowerBlanking {
    DisablePowerBlanking,
    EnablePowerBlanking,
}
impl Default for PowerBlanking {
    fn default() -> Self {
        PowerBlanking::DisablePowerBlanking
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum EchoQualificationThreshold {
    Mv35,
    Mv50,
    Mv75,
    Mv125,
    Mv220,
    Mv410,
    Mv775,
    Mv1500,
}
impl Default for EchoQualificationThreshold {
    fn default() -> Self {
        EchoQualificationThreshold::Mv125
    }
}

const RECEIVE_MODE_BIT_OFFSET: u8 = 6;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ReceiveMode {
    SingleEcho,
    MultiEcho,
}
impl Default for ReceiveMode {
    fn default() -> Self {
        ReceiveMode::SingleEcho
    }
}

const TRIGGER_EDGE_POLARITY_BIT_OFFSET: u8 = 5;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TriggerEdgePolarity {
    RisingEdge,
    FallingEdge,
}
impl Default for TriggerEdgePolarity {
    fn default() -> Self {
        TriggerEdgePolarity::RisingEdge
    }
}

pub struct TxPulseShiftPosition(u8);
impl TxPulseShiftPosition {
    pub const LOW: u8 = 0;
    pub const HIGH: u8 = 31;
    pub fn new(pulse_shift_position: u8) -> Self {
        TxPulseShiftPosition(trim_value_u8(
            pulse_shift_position,
            Self::HIGH,
            Self::LOW,
        ))
    }
    pub fn get_value(&self) -> u8 {
        self.0
    }
}
impl Default for TxPulseShiftPosition {
    fn default() -> Self {
        TxPulseShiftPosition::new(31)
    }
}

const PGA_GAIN_BIT_OFFSET: u8 = 5;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PgaGain {
    DB0,
    DB3,
    DB6,
    DB9,
    DB12,
    DB15,
    DB18,
    DB21,
}
impl Default for PgaGain {
    fn default() -> Self {
        PgaGain::DB0
    }
}

const PGA_CTRL_BIT_OFFSET: u8 = 4;
const LNA_CTRL_BIT_OFFSET: u8 = 3;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum AmplifierControl {
    Active,
    BypassedAndPoweredOff,
}
impl Default for AmplifierControl {
    fn default() -> Self {
        AmplifierControl::Active
    }
}

const LNA_FB_BIT_OFFSET: u8 = 2;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum LnaFeedbackMode {
    CapacitiveMode,
    ResistiveMode,
}
impl Default for LnaFeedbackMode {
    fn default() -> Self {
        LnaFeedbackMode::CapacitiveMode
    }
}

pub struct TimeOfFlightValue(u16);
impl TimeOfFlightValue {
    pub const LOW: u16 = 0;
    pub const HIGH: u16 = 1023;
    pub fn new(time_of_flight_value: u16) -> Self {
        TimeOfFlightValue(trim_value_u16(
            time_of_flight_value,
            Self::HIGH,
            Self::LOW,
        ))
    }
    pub fn get_2_high_bits_of_tof(&self) -> u8 {
        (self.0 >> 8) as u8
    }
    pub fn get_8_low_bits_of_tof(&self) -> u8 {
        self.0 as u8
    }
}
impl Default for TimeOfFlightValue {
    fn default() -> Self {
        TimeOfFlightValue::new(0)
    }
}

const ERR_SIG_WEAK_BIT_OFFSET: u8 = 2;
#[repr(u8)]
#[derive(PartialEq)]
pub enum ErrSignalWeakRead {
    NoError,
    SignalWeekTimeout,
}
impl Default for ErrSignalWeakRead {
    fn default() -> Self {
        ErrSignalWeakRead::NoError
    }
}

const ERR_NO_SIG_BIT_OFFSET: u8 = 1;
#[repr(u8)]
#[derive(PartialEq)]
pub enum ErrNoSignalRead {
    NoError,
    NoSignalTimeout,
}
impl Default for ErrNoSignalRead {
    fn default() -> Self {
        ErrNoSignalRead::NoError
    }
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum ErrSignalHighRead {
    NoError,
    SignalHigh,
}
impl Default for ErrSignalHighRead {
    fn default() -> Self {
        ErrSignalHighRead::NoError
    }
}

const FORCE_SHORT_TOF_BIT_OFFSET: u8 = 6;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ForceShortTimeOfFlight {
    Disabled,
    ForceShortTimeOfFlight,
}
impl Default for ForceShortTimeOfFlight {
    fn default() -> Self {
        ForceShortTimeOfFlight::Disabled
    }
}

const SHORT_TOF_BLANK_PERIOD_BIT_OFFSET: u8 = 3;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ShortTofBlankPeriod {
    T0Times8,
    T0Times16,
    T0Times32,
    T0Times64,
    T0Times128,
    T0Times256,
    T0Times512,
    T0Times1024,
}
impl Default for ShortTofBlankPeriod {
    fn default() -> Self {
        ShortTofBlankPeriod::T0Times64
    }
}

const ECHO_TIMEOUT_BIT_OFFSET: u8 = 2;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum EchoTimeout {
    EnableTimeout,
    DisableTimeout,
}
impl Default for EchoTimeout {
    fn default() -> Self {
        EchoTimeout::EnableTimeout
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TofTimeoutControl {
    T0Times128,
    T0Times256,
    T0Times512,
    T0Times1024,
}
impl Default for TofTimeoutControl {
    fn default() -> Self {
        TofTimeoutControl::T0Times256
    }
}

const CLOCK_IN_DIV_BIT_OFFSET: u8 = 2;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ClockInDiv {
    DivideBy1,
    DivideBy2,
}
impl Default for ClockInDiv {
    fn default() -> Self {
        ClockInDiv::DivideBy1
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum AutoZeroPeriod {
    T0Times64,
    T0Times128,
    T0Times256,
    T0Times512,
}
impl Default for AutoZeroPeriod {
    fn default() -> Self {
        AutoZeroPeriod::T0Times64
    }
}

#[derive(Default)]
pub struct Config0 {
    tx_frequency_divider: TxFrequencyDivider,
    tx_pulses: TxPulses,
}

#[derive(Default)]
pub struct Config1 {
    measurement_cycles: MeasurementCycles,
    receive_events_cnt: ReceiveEventsCnt,
}

#[derive(Default)]
pub struct Config2 {
    voltage_reference: VoltageReference,
    measurement_mode: MeasurementMode,
    damping_mode: DampingMode,
    channel_swap: ChannelSwap,
    ext_channel_select: ExternalChannelSelect,
    channel_select: ChannelSelect,
    tof_meas_mode: TOFMeasurementMode,
}

#[derive(Default)]
pub struct Config3 {
    temp_mode: TempMode,
    temp_rtd: TempRtdSelect,
    temp_clk_div: TempClockDivider,
    blanking: PowerBlanking,
    echo_qualification_threshold: EchoQualificationThreshold,
}

#[derive(Default)]
pub struct Config4 {
    receive_mode: ReceiveMode,
    trigger_edge_polarity: TriggerEdgePolarity,
    tx_pulse_shift_position: TxPulseShiftPosition,
}

#[derive(Default)]
pub struct AmplifierAndTimeOfFlight {
    pga_gain: PgaGain,
    pga_ctrl: AmplifierControl,
    lna_ctrl: AmplifierControl,
    lna_fb: LnaFeedbackMode,
    time_of_flight: TimeOfFlightValue,
}

#[derive(Default)]
pub struct ErrorFlagsRead {
    signal_week: ErrSignalWeakRead,
    no_signal: ErrNoSignalRead,
    signal_high: ErrSignalHighRead,
}

impl ErrorFlagsRead {
    pub fn signal_week(&self) -> &ErrSignalWeakRead {
        &self.signal_week
    }
    pub fn no_signal(&self) -> &ErrNoSignalRead {
        &self.no_signal
    }
    pub fn signal_high(&self) -> &ErrSignalHighRead {
        &self.signal_high
    }
}

#[repr(u8)]
pub enum ErrorFlagsWrite {
    DoNothing,
    ResetAllErrorFlagsAndErrorPin,
    ResetStateMachineAndMeasurement,
    ResetErrorStatemachineAndMeasurement,
}
impl Default for ErrorFlagsWrite {
    fn default() -> Self {
        ErrorFlagsWrite::DoNothing
    }
}

#[derive(Default)]
pub struct ClockRate {
    clock_in_div: ClockInDiv,
    auto_zero_period: AutoZeroPeriod,
}

#[derive(Default)]
pub struct TimeOut {
    force_short_tof: ForceShortTimeOfFlight,
    short_tof_blank_period: ShortTofBlankPeriod,
    echo_timeout: EchoTimeout,
    tof_timeout_crl: TofTimeoutControl,
}

#[derive(Default)]
pub struct Tdc1000 {
    config0: Config0,
    config1: Config1,
    config2: Config2,
    config3: Config3,
    config4: Config4,
    amplifier_and_time_of_flight: AmplifierAndTimeOfFlight,
    timeout: TimeOut,
    clock_rate: ClockRate,
}

impl Tdc1000 {
    pub fn set_tx_frequency_divider(&mut self, divider: TxFrequencyDivider) {
        self.config0.tx_frequency_divider = divider;
    }

    pub fn set_number_of_tx_pulses(&mut self, pulses: TxPulses) {
        self.config0.tx_pulses = pulses;
    }

    pub fn set_measurement_cycles(&mut self, cycles: MeasurementCycles) {
        self.config1.measurement_cycles = cycles;
    }

    pub fn set_receive_events(&mut self, events_cnt: ReceiveEventsCnt) {
        self.config1.receive_events_cnt = events_cnt;
    }

    pub fn set_common_voltage_reference_mode(
        &mut self,
        voltage_reference: VoltageReference,
    ) {
        self.config2.voltage_reference = voltage_reference;
    }

    pub fn set_measure_mode(&mut self, measure_mode: MeasurementMode) {
        self.config2.measurement_mode = measure_mode;
    }

    pub fn set_damping(&mut self, damping_mode: DampingMode) {
        self.config2.damping_mode = damping_mode;
    }

    pub fn set_channel_swap(&mut self, channel_swap: ChannelSwap) {
        self.config2.channel_swap = channel_swap;
    }

    pub fn set_external_channel_select(
        &mut self,
        external_channel_select: ExternalChannelSelect,
    ) {
        self.config2.ext_channel_select = external_channel_select;
    }

    pub fn set_active_channel(&mut self, channel: ChannelSelect) {
        self.config2.channel_select = channel;
    }

    pub fn set_tof_meas_mode(
        &mut self,
        tof_measurement_mode: TOFMeasurementMode,
    ) {
        self.config2.tof_meas_mode = tof_measurement_mode;
    }

    pub fn set_temp_measurement_mode(
        &mut self,
        temp_measurement_mode: TempMode,
    ) {
        self.config3.temp_mode = temp_measurement_mode;
    }

    pub fn set_temp_rtd_type(&mut self, temp_rtd: TempRtdSelect) {
        self.config3.temp_rtd = temp_rtd;
    }

    pub fn set_temp_clock_divider(&mut self, temp_clock_div: TempClockDivider) {
        self.config3.temp_clk_div = temp_clock_div;
    }

    pub fn set_blanking(&mut self, blanking: PowerBlanking) {
        self.config3.blanking = blanking;
    }

    pub fn set_echo_qualification_threshold(
        &mut self,
        threshold: EchoQualificationThreshold,
    ) {
        self.config3.echo_qualification_threshold = threshold;
    }

    pub fn set_receive_mode(&mut self, receive_mode: ReceiveMode) {
        self.config4.receive_mode = receive_mode;
    }

    pub fn set_trigger(&mut self, trigger: TriggerEdgePolarity) {
        self.config4.trigger_edge_polarity = trigger;
    }

    pub fn set_tx_pulse_shift_position(
        &mut self,
        position: TxPulseShiftPosition,
    ) {
        self.config4.tx_pulse_shift_position = position;
    }

    pub fn set_time_of_flight(&mut self, tof: TimeOfFlightValue) {
        self.amplifier_and_time_of_flight.time_of_flight = tof;
    }

    pub fn set_pga_gain(&mut self, gain: PgaGain) {
        self.amplifier_and_time_of_flight.pga_gain = gain;
    }

    pub fn set_pga_control(&mut self, control: AmplifierControl) {
        self.amplifier_and_time_of_flight.pga_ctrl = control;
    }

    pub fn set_lna_control(&mut self, control: AmplifierControl) {
        self.amplifier_and_time_of_flight.lna_ctrl = control;
    }

    pub fn set_lna_feedback_mode(&mut self, feedback_mode: LnaFeedbackMode) {
        self.amplifier_and_time_of_flight.lna_fb = feedback_mode;
    }

    pub fn set_tof_value(&mut self, tof_value: TimeOfFlightValue) {
        self.amplifier_and_time_of_flight.time_of_flight = tof_value;
    }

    pub fn set_force_short_tof(
        &mut self,
        force_short_tof: ForceShortTimeOfFlight,
    ) {
        self.timeout.force_short_tof = force_short_tof;
    }

    pub fn set_short_tof_blank_period(
        &mut self,
        blank_period: ShortTofBlankPeriod,
    ) {
        self.timeout.short_tof_blank_period = blank_period;
    }

    pub fn set_echo_timeout(&mut self, timeout: EchoTimeout) {
        self.timeout.echo_timeout = timeout;
    }

    pub fn set_tof_timeout_ctrl(&mut self, timeout_ctrl: TofTimeoutControl) {
        self.timeout.tof_timeout_crl = timeout_ctrl;
    }

    pub fn set_clock_in_div(&mut self, clock_in_div: ClockInDiv) {
        self.clock_rate.clock_in_div = clock_in_div;
    }

    pub fn set_auto_zero_period(&mut self, auto_zero_period: AutoZeroPeriod) {
        self.clock_rate.auto_zero_period = auto_zero_period;
    }

    pub fn get_config_0_value(&self) -> u8 {
        let tx_frequency_divider = self.config0.tx_frequency_divider as u8;
        let tx_pulses = self.config0.tx_pulses.get_value();
        tx_frequency_divider << FREQUENCY_DIVIDER_BIT_OFFSET | tx_pulses
    }

    pub fn get_config_1_value(&self) -> u8 {
        let measurement_cycles = self.config1.measurement_cycles as u8;
        let stop_pulse_count = self.config1.receive_events_cnt as u8;
        measurement_cycles << MEASUREMENT_CYCLES_BIT_OFFSET | stop_pulse_count
    }

    pub fn get_config_2_value(&self) -> u8 {
        let voltage_reference = self.config2.voltage_reference as u8;
        let measurement_mode = self.config2.measurement_mode as u8;
        let damping = self.config2.damping_mode as u8;
        let channel_swap = self.config2.channel_swap as u8;
        let ext_channel_select = self.config2.ext_channel_select as u8;
        let channel_select = self.config2.channel_select as u8;
        let tof_measurement_mode = self.config2.tof_meas_mode as u8;
        voltage_reference << VOLTAGE_REFERENCE_BIT_OFFSET
            | measurement_mode << MEASUREMENT_MODE_BIT_OFFSET
            | damping << DAMPING_MODE_BIT_OFFSET
            | channel_swap << CHANNEL_SWAP_BIT_OFFSET
            | ext_channel_select << EXTERNAL_CHANNEL_SELECT_BIT_OFFSET
            | channel_select << CHANNEL_SELECT_BIT_OFFSET
            | tof_measurement_mode
    }

    pub fn get_config_3_value(&self) -> u8 {
        let temp_mode = self.config3.temp_mode as u8;
        let temp_rtd_sel = self.config3.temp_rtd as u8;
        let temp_clk_div = self.config3.temp_clk_div as u8;
        let blanking = self.config3.blanking as u8;
        let echo_th = self.config3.echo_qualification_threshold as u8;
        temp_mode << TEMP_MODE_BIT_OFFSET
            | temp_rtd_sel << TEMP_RTD_SELECT_BIT_OFFSET
            | temp_clk_div << TEMP_CLK_DIV_BIT_OFFSET
            | blanking << BLANKING_BIT_OFFSET
            | echo_th
    }

    pub fn get_config_4_value(&self) -> u8 {
        let receive_mode = self.config4.receive_mode as u8;
        let trigger_edge_polarity = self.config4.trigger_edge_polarity as u8;
        let pulse_shift_position =
            self.config4.tx_pulse_shift_position.get_value();
        receive_mode << RECEIVE_MODE_BIT_OFFSET
            | trigger_edge_polarity << TRIGGER_EDGE_POLARITY_BIT_OFFSET
            | pulse_shift_position
    }

    pub fn get_tof_1_value(&self) -> u8 {
        let pga_gain = self.amplifier_and_time_of_flight.pga_gain as u8;
        let pga_ctrl = self.amplifier_and_time_of_flight.pga_ctrl as u8;
        let lna_ctrl = self.amplifier_and_time_of_flight.lna_ctrl as u8;
        let lna_fb = self.amplifier_and_time_of_flight.lna_fb as u8;
        let timing_reg_high = self
            .amplifier_and_time_of_flight
            .time_of_flight
            .get_2_high_bits_of_tof();
        pga_gain << PGA_GAIN_BIT_OFFSET
            | pga_ctrl << PGA_CTRL_BIT_OFFSET
            | lna_ctrl << LNA_CTRL_BIT_OFFSET
            | lna_fb << LNA_FB_BIT_OFFSET
            | timing_reg_high
    }

    pub fn get_tof_0_value(&self) -> u8 {
        self.amplifier_and_time_of_flight
            .time_of_flight
            .get_8_low_bits_of_tof()
    }

    pub fn get_timeout_value(&self) -> u8 {
        let force_short_tof = self.timeout.force_short_tof as u8;
        let short_tof_blank_period = self.timeout.short_tof_blank_period as u8;
        let echo_timeout = self.timeout.echo_timeout as u8;
        let tof_timeout_ctrl = self.timeout.tof_timeout_crl as u8;
        force_short_tof << FORCE_SHORT_TOF_BIT_OFFSET
            | short_tof_blank_period << SHORT_TOF_BLANK_PERIOD_BIT_OFFSET
            | echo_timeout << ECHO_TIMEOUT_BIT_OFFSET
            | tof_timeout_ctrl
    }

    pub fn get_clock_rate_value(&self) -> u8 {
        let clock_in_div = self.clock_rate.clock_in_div as u8;
        let auto_zero_period = self.clock_rate.auto_zero_period as u8;
        clock_in_div << CLOCK_IN_DIV_BIT_OFFSET | auto_zero_period
    }

    pub fn write_settings<CS, SPI, CsE, SpiE>(
        &mut self,
        cs: &mut CS,
        spi: &mut SPI,
    ) -> Result<(), Error<CsE, SpiE>>
    where
        CS: OutputPin<Error = CsE>,
        SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    {
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::Config0 as u8 | SPI_WRITE_BIT,
                self.get_config_0_value(),
            ],
        )?;
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::Config1 as u8 | SPI_WRITE_BIT,
                self.get_config_1_value(),
            ],
        )?;
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::Config2 as u8 | SPI_WRITE_BIT,
                self.get_config_2_value(),
            ],
        )?;
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::Config3 as u8 | SPI_WRITE_BIT,
                self.get_config_3_value(),
            ],
        )?;
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::Config4 as u8 | SPI_WRITE_BIT,
                self.get_config_4_value(),
            ],
        )?;
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::Tof1 as u8 | SPI_WRITE_BIT,
                self.get_tof_1_value(),
            ],
        )?;
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::Tof0 as u8 | SPI_WRITE_BIT,
                self.get_tof_0_value(),
            ],
        )?;
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::TimeOut as u8 | SPI_WRITE_BIT,
                self.get_timeout_value(),
            ],
        )?;
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::ClockRate as u8 | SPI_WRITE_BIT,
                self.get_clock_rate_value(),
            ],
        )?;

        Ok(())
    }

    pub fn read_error<CS, SPI, CsE, SpiE>(
        &mut self,
        cs: &mut CS,
        spi: &mut SPI,
    ) -> Result<ErrorFlagsRead, Error<CsE, SpiE>>
    where
        CS: OutputPin<Error = CsE>,
        SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    {
        let result =
            self.read_from_spi(cs, spi, ConfigAddresses::ErrFlag as u8)?;
        let mut error_flags = ErrorFlagsRead::default();
        if result & 0b1 == 0b1 {
            error_flags.signal_high = ErrSignalHighRead::SignalHigh;
        }
        if result & (0b1 << ERR_NO_SIG_BIT_OFFSET)
            == 0b1 << ERR_NO_SIG_BIT_OFFSET
        {
            error_flags.no_signal = ErrNoSignalRead::NoSignalTimeout;
        }
        if result & (0b1 << ERR_SIG_WEAK_BIT_OFFSET)
            == 0b1 << ERR_SIG_WEAK_BIT_OFFSET
        {
            error_flags.signal_week = ErrSignalWeakRead::SignalWeekTimeout;
        }
        Ok(error_flags)
    }

    pub fn reset_error<CS, SPI, CsE, SpiE>(
        &mut self,
        cs: &mut CS,
        spi: &mut SPI,
        reset_type: ErrorFlagsWrite,
    ) -> Result<(), Error<CsE, SpiE>>
    where
        CS: OutputPin<Error = CsE>,
        SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    {
        self.write_to_spi(
            cs,
            spi,
            [
                ConfigAddresses::ErrFlag as u8 | SPI_WRITE_BIT,
                reset_type as u8,
            ],
        )?;
        Ok(())
    }

    pub fn read_raw_config_values<CS, SPI, CsE, SpiE>(
        &mut self,
        cs: &mut CS,
        spi: &mut SPI,
        raw_value_buffer: &mut [u8; 10],
    ) -> Result<(), Error<CsE, SpiE>>
    where
        CS: OutputPin<Error = CsE>,
        SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    {
        raw_value_buffer[0] =
            self.read_from_spi(cs, spi, ConfigAddresses::Config0 as u8)?;
        raw_value_buffer[1] =
            self.read_from_spi(cs, spi, ConfigAddresses::Config1 as u8)?;
        raw_value_buffer[2] =
            self.read_from_spi(cs, spi, ConfigAddresses::Config2 as u8)?;
        raw_value_buffer[3] =
            self.read_from_spi(cs, spi, ConfigAddresses::Config3 as u8)?;
        raw_value_buffer[4] =
            self.read_from_spi(cs, spi, ConfigAddresses::Config4 as u8)?;
        raw_value_buffer[5] =
            self.read_from_spi(cs, spi, ConfigAddresses::Tof1 as u8)?;
        raw_value_buffer[6] =
            self.read_from_spi(cs, spi, ConfigAddresses::Tof0 as u8)?;
        raw_value_buffer[7] =
            self.read_from_spi(cs, spi, ConfigAddresses::ErrFlag as u8)?;
        raw_value_buffer[8] =
            self.read_from_spi(cs, spi, ConfigAddresses::TimeOut as u8)?;
        raw_value_buffer[9] =
            self.read_from_spi(cs, spi, ConfigAddresses::ClockRate as u8)?;
        Ok(())
    }

    fn read_from_spi<CS, SPI, CsE, SpiE>(
        &mut self,
        cs: &mut CS,
        spi: &mut SPI,
        address_to_read: u8,
    ) -> Result<u8, Error<CsE, SpiE>>
    where
        CS: OutputPin<Error = CsE>,
        SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    {
        let mut read_buffer = [address_to_read, 0xff];
        cs.set_low().map_err(Error::ChipSelectError)?;
        spi.transfer(&mut read_buffer).map_err(Error::SpiError)?;
        cs.set_high().map_err(Error::ChipSelectError)?;
        Ok(read_buffer[1])
    }

    fn write_to_spi<CS, SPI, CsE, SpiE>(
        &mut self,
        cs: &mut CS,
        spi: &mut SPI,
        data: [u8; 2],
    ) -> Result<(), Error<CsE, SpiE>>
    where
        CS: OutputPin<Error = CsE>,
        SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    {
        cs.set_low().map_err(Error::ChipSelectError)?;
        spi.write(&data).map_err(Error::SpiError)?;
        cs.set_high().map_err(Error::ChipSelectError)?;
        Ok(())
    }
}

fn trim_value_u16(value: u16, max: u16, min: u16) -> u16 {
    value.min(max).max(min)
}

fn trim_value_u8(value: u8, max: u8, min: u8) -> u8 {
    value.min(max).max(min)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use crate::{
        ChannelSwap, EchoQualificationThreshold, MeasurementCycles,
        ReceiveEventsCnt, ShortTofBlankPeriod, TOFMeasurementMode, Tdc1000,
        TimeOfFlightValue, TxFrequencyDivider, TxPulseShiftPosition, TxPulses,
        VoltageReference,
    };

    #[test]
    fn config_0_value_for_spi_is_calculated_correctly() {
        let mut tdc1000 = Tdc1000::default();
        tdc1000.set_tx_frequency_divider(TxFrequencyDivider::DivideBy8);
        tdc1000.set_number_of_tx_pulses(TxPulses::new(1));
        let config0val = tdc1000.get_config_0_value();
        assert_eq!(config0val, 65);
    }

    #[test]
    fn config_1_value_for_spi_is_calculated_correctly() {
        let mut tdc1000 = Tdc1000::default();
        tdc1000.set_measurement_cycles(MeasurementCycles::MeasurementCycles1);
        tdc1000.set_receive_events(ReceiveEventsCnt::StopEvents4);
        let config1val = tdc1000.get_config_1_value();
        assert_eq!(config1val, 4);
    }

    #[test]
    fn config_2_value_for_spi_is_calculated_correctly() {
        let mut tdc1000 = Tdc1000::default();
        let config2val = tdc1000.get_config_2_value();
        assert_eq!(config2val, 0);

        tdc1000.set_common_voltage_reference_mode(VoltageReference::External);
        tdc1000.set_channel_swap(ChannelSwap::EnableSwap);
        tdc1000.set_tof_meas_mode(TOFMeasurementMode::Mode2);
        let config2val = tdc1000.get_config_2_value();
        assert_eq!(config2val, 146);
    }

    #[test]
    fn config_3_value_for_spi_is_calculated_correctly() {
        let mut tdc1000 = Tdc1000::default();
        tdc1000
            .set_echo_qualification_threshold(EchoQualificationThreshold::Mv75);
        let config3val = tdc1000.get_config_3_value();
        assert_eq!(config3val, 2);
    }

    #[test]
    fn config_4_value_for_spi_is_calculated_correctly() {
        let mut tdc1000 = Tdc1000::default();
        tdc1000.set_tx_pulse_shift_position(TxPulseShiftPosition::new(5));
        let config4val = tdc1000.get_config_4_value();
        assert_eq!(config4val, 5);
    }

    #[test]
    fn tof_1_value_for_spi_is_calculated_correctly() {
        let mut tdc1000 = Tdc1000::default();
        tdc1000.set_time_of_flight(TimeOfFlightValue::new(
            TimeOfFlightValue::HIGH,
        ));
        let tof1val = tdc1000.get_tof_1_value();
        assert_eq!(tof1val, 3);
    }

    #[test]
    fn tof_2_value_for_spi_is_calculated_correctly() {
        let mut tdc1000 = Tdc1000::default();
        tdc1000.set_time_of_flight(TimeOfFlightValue::new(
            TimeOfFlightValue::HIGH,
        ));
        let tof2val = tdc1000.get_tof_0_value();
        assert_eq!(tof2val, 255);
    }

    #[test]
    fn timeout_value_for_spi_is_calculated_correctly() {
        let mut tdc1000 = Tdc1000::default();
        tdc1000.set_short_tof_blank_period(ShortTofBlankPeriod::T0Times32);
        let timeout_value = tdc1000.get_timeout_value();
        assert_eq!(timeout_value, 17);
    }

    #[test]
    fn clock_rate_value_for_spi_is_calculated_correctly() {
        let tdc1000 = Tdc1000::default();
        let clock_rate_value = tdc1000.get_clock_rate_value();
        assert_eq!(clock_rate_value, 0);
    }
}
