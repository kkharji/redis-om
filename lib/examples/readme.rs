// Copyright (c) 2023 kkharji
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
use redis_om::HashModel;

#[derive(HashModel, PartialEq, Eq, Default)]
struct Customer {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub age: u32,
    pub bio: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _customer = Customer::default();

    Ok(())
}
