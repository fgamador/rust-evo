use std::convert::TryInto;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::u32;

#[derive(Debug)]
pub struct Clouds<C: Cloud> {
    clouds: Vec<C>,
}

impl<C: Cloud> Clouds<C> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Clouds { clouds: vec![] }
    }

    pub fn add_cloud(&mut self, mut cloud: C) -> CloudHandle {
        let handle = self.next_handle();
        cloud.cloud_data_mut().handle = handle;
        self.clouds.push(cloud);
        handle
    }

    fn next_handle(&self) -> CloudHandle {
        CloudHandle::new(self.clouds.len().try_into().unwrap())
    }

    pub fn is_valid_handle(&self, handle: CloudHandle) -> bool {
        (handle.index as usize) < self.clouds.len()
    }

    /// Removes the clouds referenced by `handles`.
    ///
    /// Warning: this function has two big gotchas:
    ///
    /// 1) `handles` should be in ascending order of `index`. If not, the function will
    /// panic on index out-of-bounds if we're removing clouds at the end of self.clouds.
    ///
    /// 2) Worse, this function changes the clouds referenced by some of the remaining handles.
    /// Never retain handles across a call to this function.
    pub fn remove_clouds(&mut self, handles: &[CloudHandle]) {
        for handle in handles.iter().rev() {
            self.remove_cloud(*handle);
        }
    }

    /// Warning: invalidates handles to the last cloud in self.clouds.
    fn remove_cloud(&mut self, handle: CloudHandle) {
        self.clouds.swap_remove(handle.index());
        self.fix_swapped_cloud_if_needed(handle);
    }

    fn fix_swapped_cloud_if_needed(&mut self, handle: CloudHandle) {
        let old_last_handle = self.next_handle();
        if handle != old_last_handle {
            self.cloud_mut(handle).cloud_data_mut().handle = handle;
        }
    }

    pub fn clouds(&self) -> &[C] {
        &self.clouds
    }

    pub fn clouds_mut(&mut self) -> &mut [C] {
        &mut self.clouds
    }

    pub fn cloud(&self, handle: CloudHandle) -> &C {
        &self.clouds[handle.index()]
    }

    pub fn cloud_mut(&mut self, handle: CloudHandle) -> &mut C {
        &mut self.clouds[handle.index()]
    }
}

pub trait Cloud {
    fn cloud_handle(&self) -> CloudHandle;

    fn cloud_data(&self) -> &CloudData;

    fn cloud_data_mut(&mut self) -> &mut CloudData;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CloudHandle {
    index: u32,
}

impl CloudHandle {
    fn new(index: u32) -> Self {
        CloudHandle { index }
    }

    pub fn unset() -> Self {
        CloudHandle { index: u32::MAX }
    }

    pub fn resolve<'a, C>(&self, clouds: &'a mut [C]) -> &'a C
    where
        C: Cloud,
    {
        &clouds[self.index()]
    }

    pub fn resolve_mut<'a, C>(&self, clouds: &'a mut [C]) -> &'a mut C
    where
        C: Cloud,
    {
        &mut clouds[self.index()]
    }

    fn index(self) -> usize {
        self.index as usize
    }
}

impl fmt::Display for CloudHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CloudData {
    handle: CloudHandle,
}

impl CloudData {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        CloudData {
            handle: CloudHandle::unset(),
        }
    }

    pub fn handle(&self) -> CloudHandle {
        self.handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_cloud_has_correct_handle() {
        let mut clouds = Clouds::new();

        let handle = clouds.add_cloud(SimpleCloud::new(0));

        let cloud = &clouds.clouds()[0];
        assert_eq!(cloud.cloud_handle(), handle);
    }

    #[test]
    fn can_fetch_cloud_by_handle() {
        let mut clouds = Clouds::new();

        let cloud_handle = clouds.add_cloud(SimpleCloud::new(0));

        let cloud = &clouds.clouds()[0];
        assert_eq!(*clouds.cloud(cloud_handle), *cloud);
    }

    #[test]
    fn can_remove_last_and_non_last_clouds() {
        let mut clouds = Clouds::new();
        let cloud0_handle = clouds.add_cloud(SimpleCloud::new(0));
        let _cloud1_handle = clouds.add_cloud(SimpleCloud::new(1));
        let cloud2_handle = clouds.add_cloud(SimpleCloud::new(2));

        clouds.remove_clouds(&vec![cloud0_handle, cloud2_handle]);

        assert_eq!(clouds.clouds.len(), 1);
        let cloud = &clouds.clouds()[0];
        assert_eq!(cloud.id, 1);
        assert_eq!(cloud.cloud_handle().index, 0);
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SimpleCloud {
        cloud_data: CloudData,
        pub id: i32,
    }

    impl SimpleCloud {
        pub fn new(id: i32) -> Self {
            SimpleCloud {
                cloud_data: CloudData::new(),
                id,
            }
        }
    }

    impl Cloud for SimpleCloud {
        fn cloud_handle(&self) -> CloudHandle {
            self.cloud_data.handle
        }

        fn cloud_data(&self) -> &CloudData {
            &self.cloud_data
        }

        fn cloud_data_mut(&mut self) -> &mut CloudData {
            &mut self.cloud_data
        }
    }
}
