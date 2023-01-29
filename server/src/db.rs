use std::{any::type_name, fmt::Debug, marker::PhantomData, sync::Arc};

use log::{info, trace};
use serde::{de::DeserializeOwned, Serialize};

use crate::User;

pub struct Db<K: AsRef<[u8]> + Debug + ?Sized, T: Serialize + DeserializeOwned>(
    Arc<sled::Db>,
    PhantomData<K>,
    PhantomData<T>,
);

impl<K: AsRef<[u8]> + Debug + ?Sized, T: Serialize + DeserializeOwned> Clone for Db<K, T> {
    fn clone(&self) -> Self {
        Db(Arc::clone(&self.0), PhantomData, PhantomData)
    }
}

impl<K: AsRef<[u8]> + Debug + ?Sized, T: Serialize + DeserializeOwned> Db<K, T> {
    pub fn open(string: &str) -> Db<K, T> {
        info!("Opening {} DB from {string}", type_name::<T>());

        Db(
            Arc::new(sled::open(string).expect("the database to be available")),
            PhantomData,
            PhantomData,
        )
    }

    pub fn contains(&self, key: &K) -> Result<bool, anyhow::Error> {
        trace!(
            "Checking if {key:?} exists in {} database",
            type_name::<T>()
        );

        Ok(self.0.contains_key(key)?)
    }

    pub fn add(&self, key: &K, val: &T) -> Result<(), anyhow::Error> {
        trace!("Adding `{key:?}` to the {} database", type_name::<T>());

        self.0.insert(key, serde_json::to_vec(val)?)?;

        Ok(())
    }

    pub fn get(&self, key: &K) -> Result<Option<User>, anyhow::Error> {
        trace!("Getting `{key:?}` from the {} database", type_name::<T>());

        match self.0.get(key)? {
            Some(v) => Ok(serde_json::from_slice(&v)?),
            None => Ok(None),
        }
    }
}
