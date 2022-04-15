use crate::constants::*;

type Region<T> = [T; (REGION_SIZE * REGION_SIZE) as usize];

pub struct RegionCache<T> {
    default_value: T,
    regions: Vec<Option<Box<Region<T>>>>,
}

impl<T: Clone + Default> Default for RegionCache<T> {
    fn default() -> RegionCache<T> {
        RegionCache {
            default_value: Default::default(),
            regions: vec![None; (HORIZONTAL_REGIONS * VERTICAL_REGIONS * PLANES) as usize],
        }
    }
}

impl<T: Copy> RegionCache<T> {
    pub fn new(default_value: T) -> RegionCache<T> {
        RegionCache {
            default_value,
            regions: vec![None; (HORIZONTAL_REGIONS * VERTICAL_REGIONS * PLANES) as usize],
        }
    }

    /*
     * Can't use IndexMut trait because Index trait is implemented on immutable self,
     * can't allocate regions. RefCell possible solution, but overhead is nonsensical.
    */
    pub fn get_mut(&mut self, index: u32) -> &mut T {
        let x = index % WIDTH;
        let y = index / WIDTH;
        let region_index = (y / REGION_SIZE) * HORIZONTAL_REGIONS + x / REGION_SIZE;
        let region = self.regions[region_index as usize]
            .get_or_insert_with(|| Box::new([self.default_value; (REGION_SIZE * REGION_SIZE) as usize]));
        &mut region[((y % REGION_SIZE) * REGION_SIZE + x % REGION_SIZE) as usize]
    }

    pub fn get(&self, index: u32) -> Option<&T> {
        let x = index % WIDTH;
        let y = index / WIDTH;
        let region_index = (y / REGION_SIZE) * HORIZONTAL_REGIONS + x / REGION_SIZE;
        let region = self.regions[region_index as usize].as_ref()?;
        Some(&region[((y % REGION_SIZE) * REGION_SIZE + x % REGION_SIZE) as usize])
    }

    pub fn mem_usage(&self) -> usize {
        self.regions.iter().map(|v| {
            std::mem::size_of_val(v) + if v.is_some() { std::mem::size_of::<Region<T>>() } else { 0 }
        }).sum()
    }
}
