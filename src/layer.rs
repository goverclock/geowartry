/// z value for transforms
#[derive(PartialEq)]
pub enum Layer {
    SelectArea,
    Units,
    GameMap,
}

impl Layer {
    fn iter() -> core::array::IntoIter<Layer, 3> {
        [Layer::SelectArea, Layer::Units, Layer::GameMap].into_iter()
    }

    pub fn into_z_value(self) -> f32 {
        let it = Self::iter();
        let mut z = 100.0;
        for i in it {
            if i == self {
                return z;
            }
            z -= 1.0
        }
        unreachable!()
    }
}
