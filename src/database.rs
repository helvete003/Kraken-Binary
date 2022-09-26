use mysql::*;
use mysql::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Customer {
    pub id: Option<u32>,
    pub kunde: Option<String>,
    pub url: Option<String>,
    pub password: Option<String>
}


pub struct Database {
    connection: Conn
}

impl Database {
    pub fn new(login_string: &String) -> Result<Self> {
        let url = Opts::from_url(login_string)?;
        match Conn::new(url) {
            Ok(connection) => Ok(Self {
                connection
            }),
            Err(err) => Err(err),
        }
    }
    pub fn get_customer_data(&mut self) -> Result<Vec<Customer>> {
        self.connection
            .query_map(
                "SELECT id, kunde, pk_url AS url,pk_password AS password FROM hostings WHERE pk_url<>'' AND pk_password<>'' ORDER BY kunde ASC",
                |(id, kunde, url, password)| {
                    Customer {id, kunde, url, password }
                }
            )
    }
}