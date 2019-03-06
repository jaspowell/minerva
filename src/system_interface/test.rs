// Copyright (c) 2019 Decode Detroit
// Author: Patton Doyle
// Licence: GNU GPLv3
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! A testing module that implements useful tools for the testing the program.


/// Test_Vec Macro
///
/// A macro that allows easier comparison of two vecotrs (one the test vector
/// and the other generated by the test itself).
///
#[cfg(test)]
macro_rules! test_vec {

    // Compare the test vector with the messages received (order matters)
    (=$line:expr, $test:expr) => ({
        
        // Import necessary libraries
        use std::thread;
        use std::time::Duration;
        
        // Print and check the messages received (wait at most half a second)
        let mut index = 0;
        let mut recv = Vec::new();
        while index < 500 {

            // Try to find the five test updates
            match $line.try_recv() {
            
                // Log the new addition and check for all of them
                Ok(message) => {
                    recv.push(message.clone());
                    println!("{}", message);
                    
                    // Check that the received vector matches the test vector
                    if $test == recv {
                        return
                    }
                }
                
                // Do nothing if nothing is received
                Err(_) => (),
            }
            
            // Increment and give up after half a second
            thread::sleep(Duration::from_millis(1));
            index = index + 1;
        }
        
        // Print debugging help if the script failed
        println!("===================DEBUG==================\n\nEXPECTED\n{:?}\n\nOUTPUT\n{:?}", $test, recv);
        
        // If they were not found, fail the test
        panic!("Failed test vector comparison.");
    });
    
    // Compare the test vector with the messages received (order irrelevant)
    (~$line:expr, $test:expr) => ({
        
        // Import necessary libraries
        use std::thread;
        use std::time::Duration;
        
        // Print and check the messages received (wait at most half a second)
        let mut index = 0;
        let mut recv = Vec::new();
        while index < 500 {

            // Try to find the five test updates
            match $line.try_recv() {
            
                // Log the new addition and check for all of them
                Ok(message) => {
                    recv.push(message.clone());
                    println!("{}", message);
                    
                    // Check that the received vector matches the test vector (order irrelevant)
                    let mut i = 0;
                    for item in $test.iter() {
                        
                        // Check that all elements are satisfied
                        if recv.contains(item) {
                            i = i + 1;
                            
                            // Return if/when all test items are found
                            if i >= $test.len() {
                                return
                            }
                        }
                    }
                }
                
                // Do nothing if nothing is received
                Err(_) => (),
            }
            
            // Increment and give up after half a second
            thread::sleep(Duration::from_millis(1));
            index = index + 1;
        }
        
        // Print debugging help if the script failed
        println!("===================DEBUG==================\n\nEXPECTED\n{:?}\n\nOUTPUT\n{:?}", $test, recv);
        
        // If they were not found, fail the test
        panic!("Failed test vector comparison.");
    });
}

