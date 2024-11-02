use super::peripherals::Peripheral;

pub trait UseOfExternalPeripheralsExt {
    fn register_external_peripherals_use(
        &mut self,
        peripherals: Vec<Peripheral>,
    ) -> Vec<Peripheral>;
    fn register_external_peripheral_use(&mut self, peripheral: Peripheral) -> Peripheral;
}
