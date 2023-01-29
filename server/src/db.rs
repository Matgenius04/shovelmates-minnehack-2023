use std::{any::type_name, marker::PhantomData, sync::Arc};

use log::{info, trace};
use serde::{de::DeserializeOwned, Serialize};

pub struct Db<T: Serialize + DeserializeOwned>(Arc<sled::Db>, PhantomData<T>);

impl<T: Serialize + DeserializeOwned> Clone for Db<T> {
    fn clone(&self) -> Self {
        Db(Arc::clone(&self.0), PhantomData)
    }
}

impl<T: Serialize + DeserializeOwned> Db<T> {
    pub fn open(string: &str) -> Db<T> {
        info!("Opening {} DB from {string}", type_name::<T>());

        Db(
            Arc::new(sled::open(string).expect("the database to be available")),
            PhantomData,
        )
    }

    pub fn contains(&self, key: &str) -> Result<bool, anyhow::Error> {
        trace!("Checking if {key} exists in {} database", type_name::<T>());

        Ok(self.0.contains_key(key)?)
    }

    pub fn add(&self, key: &str, val: &T) -> Result<(), anyhow::Error> {
        trace!("Adding `{key}` to the {} database", type_name::<T>());

        self.0.insert(key, serde_json::to_vec(val)?)?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<T>, anyhow::Error> {
        trace!("Getting `{key}` from the {} database", type_name::<T>());

        match self.0.get(key)? {
            Some(v) => Ok(serde_json::from_slice(&v)?),
            None => Ok(None),
        }
    }

    pub fn delete(&self, key: &str) -> Result<Option<T>, anyhow::Error> {
        trace!("Deleting {key} from the {} database", type_name::<T>());

        match self.0.remove(key)? {
            Some(v) => Ok(serde_json::from_slice(&v)?),
            None => Ok(None),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Result<(String, T), anyhow::Error>> {
        self.0.iter().map(|maybe_v| {
            let (key, val) = maybe_v?;

            let str = String::from_utf8(key.to_vec())?;
            let t = serde_json::from_slice(&val)?;

            Ok((str, t))
        })
    }
}
