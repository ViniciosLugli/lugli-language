/// User-focused scenario tests that validate real-world usage patterns
/// These tests identify gaps and validate practical programming scenarios
use lugli_common::Value;
use lugli_vm::Vm;

fn run_code(source: &str) -> Result<Value, String> {
    let (program, span_map) = lugli_parser::parse(source).map_err(|e| e.to_string())?;
    {
        let mut vm = Vm::new();
        vm.compile_and_run(&program, span_map)
    }
    .map_err(|e| e.to_string())
}

fn run_and_get_number(source: &str) -> Result<f64, String> {
    match run_code(source)? {
        Value::Number(n) => Ok(n),
        other => Err(format!("Expected number, got {}", other.type_name())),
    }
}

fn _run_and_get_bool(source: &str) -> Result<bool, String> {
    match run_code(source)? {
        Value::Bool(b) => Ok(b),
        other => Err(format!("Expected bool, got {}", other.type_name())),
    }
}

#[cfg(test)]
mod validation_patterns {
    use super::*;

    #[test]
    fn test_input_validation_chain() {
        // Real-world scenario: validating user input with multiple checks
        let source = r#"
            fn validate_email(email) {
                if len(email) == 0 {
                    return "Email cannot be empty"
                }

                if !email.contains("@") {
                    return "Email must contain @"
                }

                if len(email) < 5 {
                    return "Email too short"
                }

                return "valid"
            }

            let r1 = validate_email("")
            let r2 = validate_email("test")
            let r3 = validate_email("a@b")
            let r4 = validate_email("valid@email.com")

            if r1 == "Email cannot be empty" && r2 == "Email must contain @" &&
               r3 == "Email too short" && r4 == "valid" {
                1
            } else {
                0
            }
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_password_strength_validator() {
        // Practical password validation with multiple criteria
        let source = r#"
            fn check_password_strength(password) {
                mut score = 0

                if len(password) >= 8 {
                    score = score + 1
                }

                if len(password) >= 12 {
                    score = score + 1
                }

                # Check for digit (simplified - just checking if contains specific digits)
                if password.contains("0") || password.contains("1") ||
                   password.contains("2") || password.contains("3") {
                    score = score + 1
                }

                return score
            }

            check_password_strength("ab12cdefgh")
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 2.0); // Length >= 8 (1) + contains digit (1) = 2
    }

    #[test]
    fn test_range_validation() {
        // Common pattern: validating numbers are in range
        let source = r#"
            fn validate_age(age) {
                if age < 0 {
                    return false
                }
                if age > 150 {
                    return false
                }
                return true
            }

            let tests = [
                validate_age(-5),
                validate_age(0),
                validate_age(25),
                validate_age(150),
                validate_age(200)
            ]

            # Count valid results (false, true, true, true, false)
            mut count = 0
            for result in tests {
                if result {
                    count = count + 1
                }
            }
            count
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 3.0); // 3 valid ages
    }
}

#[cfg(test)]
mod data_transformation {
    use super::*;

    #[test]
    fn test_map_reduce_pattern() {
        // Classic map-reduce: transform then aggregate
        let source = r#"
            let numbers = [1, 2, 3, 4, 5]

            # Map: square each number
            let squared = [x * x for x in numbers]

            # Reduce: sum all squares
            mut sum = 0
            for n in squared {
                sum = sum + n
            }

            sum
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 55.0); // 1 + 4 + 9 + 16 + 25 = 55
    }

    #[test]
    fn test_group_by_pattern() {
        // Common scenario: grouping data by a property
        let source = r#"
            let people = [
                {"name": "Alice", "city": "NYC", "age": 30},
                {"name": "Bob", "city": "LA", "age": 25},
                {"name": "Charlie", "city": "NYC", "age": 35},
                {"name": "David", "city": "LA", "age": 28}
            ]

            # Group by city (manual since we don't have groupBy function)
            let nyc_people = []
            let la_people = []

            for person in people {
                if person["city"] == "NYC" {
                    nyc_people.push(person)
                } elif person["city"] == "LA" {
                    la_people.push(person)
                }
            }

            len(nyc_people) * 10 + len(la_people)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 22.0); // 2 NYC + 2 LA = 22
    }

    #[test]
    fn test_filter_and_transform_pipeline() {
        // Real pipeline: filter -> transform -> aggregate
        let source = r#"
            let products = [
                {"name": "Apple", "price": 1.5, "category": "fruit"},
                {"name": "Bread", "price": 2.5, "category": "bakery"},
                {"name": "Banana", "price": 0.8, "category": "fruit"},
                {"name": "Cake", "price": 5.0, "category": "bakery"}
            ]

            # Filter fruits only
            let fruits = [p for p in products if p["category"] == "fruit"]

            # Calculate total fruit cost
            mut total = 0
            for fruit in fruits {
                total = total + fruit["price"]
            }

            total
        "#;

        let result = run_and_get_number(source).unwrap();
        assert!((result - 2.3).abs() < 0.01); // 1.5 + 0.8 = 2.3
    }

    #[test]
    fn test_nested_data_extraction() {
        // Extracting data from nested structures
        let source = r#"
            let company = {
                "name": "TechCorp",
                "departments": [
                    {"name": "Engineering", "employees": 50},
                    {"name": "Sales", "employees": 30},
                    {"name": "Marketing", "employees": 20}
                ]
            }

            # Calculate total employees
            mut total = 0
            for dept in company["departments"] {
                total = total + dept["employees"]
            }

            total
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 100.0);
    }
}

#[cfg(test)]
mod algorithm_implementations {
    use super::*;

    #[test]
    fn test_bubble_sort() {
        // Common algorithm: bubble sort
        let source = r#"
            fn bubble_sort(arr) {
                let n = len(arr)

                for i in range(n) {
                    for j in range(0, n - i - 1) {
                        if arr[j] > arr[j + 1] {
                            # Swap
                            let temp = arr[j]
                            arr[j] = arr[j + 1]
                            arr[j + 1] = temp
                        }
                    }
                }

                return arr
            }

            let unsorted = [64, 34, 25, 12, 22, 11, 90]
            let sorted = bubble_sort(unsorted)

            # Return first element (should be 11)
            sorted[0]
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 11.0);
    }

    #[test]
    fn test_binary_search() {
        // Binary search on sorted array
        let source = r#"
            fn binary_search(arr, target) {
                mut left = 0
                mut right = len(arr) - 1

                loop {
                    if left > right {
                        return -1
                    }

                    let mid = (left + right) // 2  # Integer division
                    let mid_val = arr[mid]

                    if mid_val == target {
                        return mid
                    } elif mid_val < target {
                        left = mid + 1
                    } else {
                        right = mid - 1
                    }
                }
            }

            let sorted = [1, 3, 5, 7, 9, 11, 13, 15]
            binary_search(sorted, 7)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 3.0); // Index 3
    }

    #[test]
    fn test_find_duplicates() {
        // Find duplicate elements in array
        let source = r#"
            fn find_duplicates(arr) {
                let seen = {}
                let duplicates = []

                for item in arr {
                    if seen.has_key(item) {
                        # Only add to duplicates if not already there
                        mut found = false
                        for dup in duplicates {
                            if dup == item {
                                found = true
                            }
                        }
                        if !found {
                            duplicates.push(item)
                        }
                    } else {
                        seen[item] = true
                    }
                }

                return duplicates
            }

            let numbers = [1, 2, 3, 2, 4, 5, 3, 6]
            let dups = find_duplicates(numbers)
            len(dups)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 2.0); // 2 and 3 are duplicates
    }

    #[test]
    fn test_palindrome_checker() {
        // Check if string is palindrome
        let source = r#"
            fn is_palindrome(text) {
                let n = len(text)
                let half = n / 2

                for i in range(half) {
                    if text[i] != text[n - 1 - i] {
                        return false
                    }
                }

                return true
            }

            let t1 = is_palindrome("racecar")
            let t2 = is_palindrome("hello")
            let t3 = is_palindrome("a")

            if t1 && !t2 && t3 {
                1
            } else {
                0
            }
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1.0);
    }
}

#[cfg(test)]
mod state_machines {
    use super::*;

    #[test]
    fn test_traffic_light_state_machine() {
        // Simulating a traffic light with state transitions
        let source = r#"
            struct TrafficLight {
                state

                fn next(self) {
                    self.state = match self.state {
                        "red" => "green",
                        "green" => "yellow",
                        "yellow" => "red",
                        _ => "red"
                    }
                    return self
                }

                fn get_state(self) {
                    return self.state
                }
            }

            let light = TrafficLight { state: "red" }
            light.next() # red -> green
            light.next() # green -> yellow
            light.next() # yellow -> red

            if light.get_state() == "red" {
                1
            } else {
                0
            }
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_order_status_workflow() {
        // Order processing workflow
        let source = r#"
            fn process_order(order) {
                match order["status"] {
                    "pending" => {
                        if order["payment"] {
                            order["status"] = "processing"
                        } else {
                            order["status"] = "payment_required"
                        }
                    },
                    "processing" => {
                        if order["stock"] {
                            order["status"] = "shipped"
                        } else {
                            order["status"] = "out_of_stock"
                        }
                    },
                    "shipped" => {
                        order["status"] = "delivered"
                    },
                    _ => {}
                }
                return order
            }

            let order = {"status": "pending", "payment": true, "stock": true}
            process_order(order) # pending -> processing
            process_order(order) # processing -> shipped

            if order["status"] == "shipped" {
                1
            } else {
                0
            }
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1.0);
    }
}

#[cfg(test)]
mod builder_patterns {
    use super::*;

    #[test]
    fn test_query_builder() {
        // SQL-like query builder pattern - now with keyword method names!
        let source = r#"
            struct QueryBuilder {
                table
                fields
                conditions

                fn select(self, fields) {
                    self.fields = fields
                    return self
                }

                fn from(self, table) {
                    self.table = table
                    return self
                }

                fn where(self, condition) {
                    self.conditions = condition
                    return self
                }

                fn build(self) {
                    let query = f"SELECT {self.fields} FROM {self.table}"
                    if self.conditions != "" {
                        query = f"{query} WHERE {self.conditions}"
                    }
                    return query
                }
            }

            let qb = QueryBuilder { table: "", fields: "", conditions: "" }
            let query = qb.select("*").from("users").where("age > 18").build()

            if query.contains("SELECT") && query.contains("FROM users") && query.contains("WHERE") {
                1
            } else {
                0
            }
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_http_request_builder() {
        // HTTP request builder with method chaining
        let source = r#"
            struct RequestBuilder {
                url
                method
                headers

                fn set_url(self, url) {
                    self.url = url
                    return self
                }

                fn set_method(self, method) {
                    self.method = method
                    return self
                }

                fn add_header(self, key, value) {
                    self.headers[key] = value
                    return self
                }

                fn get_url(self) {
                    return self.url
                }
            }

            let req = RequestBuilder { url: "", method: "GET", headers: {} }
            req.set_url("https://api.com").set_method("POST").add_header("Content-Type", "application/json")

            if req.get_url() == "https://api.com" {
                1
            } else {
                0
            }
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1.0);
    }
}

#[cfg(test)]
mod caching_patterns {
    use super::*;

    #[test]
    fn test_lru_cache_simulation() {
        // Simple LRU cache concept
        let source = r#"
            struct Cache {
                data
                capacity

                fn get(self, key) {
                    if self.data.has_key(key) {
                        return self.data[key]
                    }
                    return null
                }

                fn put(self, key, value) {
                    # Simplified - doesn't actually implement LRU eviction
                    # Just demonstrates the interface
                    if len(self.data.keys()) >= self.capacity {
                        # Would evict least recently used here
                    }
                    self.data[key] = value
                    return self
                }
            }

            let cache = Cache { data: {}, capacity: 3 }
            cache.put("a", 1)
            cache.put("b", 2)
            cache.put("c", 3)

            let val = cache.get("a")
            if val == 1 {
                1
            } else {
                0
            }
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_memoization_with_expiry() {
        // Cache with simple expiry simulation
        let source = r#"
            fn make_cached_fn() {
                let cache = {}
                mut call_count = 0

                fn expensive_calc(n) {
                    call_count = call_count + 1

                    if cache.has_key(n) {
                        return cache[n]
                    }

                    # Simulate expensive calculation
                    let result = n * n
                    cache[n] = result
                    return result
                }

                return expensive_calc
            }

            let cached_fn = make_cached_fn()
            let r1 = cached_fn(5)  # Calculates
            let r2 = cached_fn(5)  # From cache
            let r3 = cached_fn(10) # Calculates

            r1 + r2 + r3
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 150.0); // 25 + 25 + 100
    }
}

#[cfg(test)]
mod error_handling_patterns {
    use super::*;

    #[test]
    fn test_try_catch_simulation() {
        // Simulating error handling with return codes
        let source = r#"
            fn safe_divide(a, b) {
                if b == 0 {
                    return {"error": true, "message": "Division by zero"}
                }
                return {"error": false, "result": a / b}
            }

            let r1 = safe_divide(10, 2)
            let r2 = safe_divide(10, 0)

            mut success_count = 0
            if !r1["error"] {
                success_count = success_count + 1
            }
            if r2["error"] {
                success_count = success_count + 1
            }

            success_count
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_validation_with_early_return() {
        // Early return pattern for validation
        let source = r#"
            fn validate_user(user) {
                if !user.has_key("name") {
                    return "Missing name"
                }

                if !user.has_key("email") {
                    return "Missing email"
                }

                if !user.has_key("age") {
                    return "Missing age"
                }

                if user["age"] < 18 {
                    return "Too young"
                }

                return "valid"
            }

            let u1 = {"name": "Alice"}
            let u2 = {"name": "Bob", "email": "bob@test.com"}
            let u3 = {"name": "Charlie", "email": "c@test.com", "age": 16}
            let u4 = {"name": "David", "email": "d@test.com", "age": 25}

            let r1 = validate_user(u1)
            let r2 = validate_user(u2)
            let r3 = validate_user(u3)
            let r4 = validate_user(u4)

            if r1 == "Missing email" && r2 == "Missing age" &&
               r3 == "Too young" && r4 == "valid" {
                1
            } else {
                0
            }
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1.0);
    }
}

#[cfg(test)]
mod functional_patterns {
    use super::*;

    #[test]
    fn test_function_composition() {
        // Composing functions manually
        let source = r#"
            fn add_one(x) { return x + 1 }
            fn double(x) { return x * 2 }
            fn square(x) { return x * x }

            fn compose(f, g) {
                fn composed(x) {
                    return f(g(x))
                }
                return composed
            }

            let add_then_double = compose(double, add_one)
            add_then_double(5)  # (5 + 1) * 2 = 12
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 12.0);
    }

    #[test]
    fn test_currying_simulation() {
        // Simulating currying
        let source = r#"
            fn make_adder(x) {
                fn adder(y) {
                    return x + y
                }
                return adder
            }

            let add5 = make_adder(5)
            let add10 = make_adder(10)

            add5(3) + add10(7)  # 8 + 17 = 25
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 25.0);
    }

    #[test]
    fn test_filter_map_reduce_chain() {
        // Full functional chain
        let source = r#"
            let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

            # Filter: only even numbers
            let evens = [x for x in numbers if x % 2 == 0]

            # Map: square them
            let squared = [x * x for x in evens]

            # Reduce: sum them
            mut sum = 0
            for n in squared {
                sum = sum + n
            }

            sum  # 4 + 16 + 36 + 64 + 100 = 220
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 220.0);
    }
}
