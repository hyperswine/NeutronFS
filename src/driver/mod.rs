// ------------------
// NEUTRON FILESYSTEM
// ------------------

/*
This is the greatest filesystem to ever exist
Example fs in example_fs. If single user, then /home is the only home
If "multiuser", other homes are in /home/guest/<name>
*/

// -------------
// API
// -------------

pub mod block;
pub mod ram;
pub mod neutronfs;
