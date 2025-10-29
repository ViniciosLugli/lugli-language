// Struct tests - validates struct functionality
//
// Tests struct declarations, instantiation, methods, and field access:
// - Basic struct definition and instantiation
// - Struct methods (self parameter)
// - Mutating methods with ! suffix
// - Field access and modification
// - Nested structs
// - Structs in collections
// - Default field values
// - Method chaining

mod helpers;
use helpers::run_test;

#[test]
fn test_struct_declaration_and_instantiation() {
    let source = r#"
        struct Person {
            name: ""
            age: 0
        }

        let alice = Person { name: "Alice", age: 30 }

        if alice.name != "Alice" {
            let error = 1 / 0  # Struct field 'name' incorrect
        }

        if alice.age != 30 {
            let error = 1 / 0  # Struct field 'age' incorrect
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_with_methods() {
    let source = r#"
        struct Rectangle {
            width: 0
            height: 0

            fn area(self) {
                return self.width * self.height
            }

            fn perimeter(self) {
                return 2 * (self.width + self.height)
            }

            fn set_dimensions!(self, w, h) {
                self.width = w
                self.height = h
            }
        }

        let rect = Rectangle { width: 10, height: 5 }

        if rect.area() != 50 {
            let error = 1 / 0  # Struct method 'area' failed
        }

        if rect.perimeter() != 30 {
            let error = 1 / 0  # Struct method 'perimeter' failed
        }

        rect.set_dimensions!(20, 10)
        if rect.area() != 200 {
            let error = 1 / 0  # Mutating struct method failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_field_modification() {
    let source = r#"
        struct Counter {
            count: 0

            fn increment!(self) {
                self.count = self.count + 1
            }

            fn get_count(self) {
                return self.count
            }
        }

        let counter = Counter { count: 0 }
        counter.increment!()
        counter.increment!()
        counter.increment!()

        if counter.get_count() != 3 {
            let error = 1 / 0  # Struct field modification failed
        }

        # Direct field modification
        counter.count = 10
        if counter.count != 10 {
            let error = 1 / 0  # Direct field assignment failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_nested_structs() {
    let source = r#"
        struct Address {
            street: ""
            city: ""
        }

        struct Person {
            name: ""
            address: null

            fn get_city(self) {
                return self.address.city
            }
        }

        let addr = Address { street: "123 Main St", city: "Springfield" }
        let person = Person { name: "Bob", address: addr }

        if person.address.street != "123 Main St" {
            let error = 1 / 0  # Nested struct field access failed
        }

        if person.get_city() != "Springfield" {
            let error = 1 / 0  # Method accessing nested struct failed
        }

        # Modify nested struct
        person.address.city = "Shelbyville"
        if person.address.city != "Shelbyville" {
            let error = 1 / 0  # Nested struct modification failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_structs_in_collections() {
    let source = r#"
        struct Point {
            x: 0
            y: 0

            fn distance_from_origin(self) {
                return (self.x * self.x + self.y * self.y) ** 0.5
            }
        }

        let points = [
            Point { x: 3, y: 4 },
            Point { x: 0, y: 0 },
            Point { x: 5, y: 12 }
        ]

        if points[0].distance_from_origin() != 5 {
            let error = 1 / 0  # Struct in array failed
        }

        # Structs in dictionary
        let named_points = {
            "origin": Point { x: 0, y: 0 },
            "unit": Point { x: 1, y: 1 }
        }

        if named_points.get("origin").x != 0 {
            let error = 1 / 0  # Struct in dictionary failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_default_values() {
    let source = r#"
        struct Config {
            host: "localhost"
            port: 8080
            debug: false
        }

        # Use all defaults
        let config1 = Config {}

        if config1.host != "localhost" {
            let error = 1 / 0  # Default string value failed
        }

        if config1.port != 8080 {
            let error = 1 / 0  # Default number value failed
        }

        if config1.debug != false {
            let error = 1 / 0  # Default boolean value failed
        }

        # Override some defaults
        let config2 = Config { port: 3000, debug: true }

        if config2.host != "localhost" {
            let error = 1 / 0  # Unspecified default failed
        }

        if config2.port != 3000 {
            let error = 1 / 0  # Overridden value failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_method_chaining() {
    let source = r#"
        struct Builder {
            value: ""

            fn add!(self, text) {
                self.value = self.value + text
                return self
            }

            fn build(self) {
                return self.value
            }
        }

        let builder = Builder { value: "" }
        let result = builder.add!("Hello").add!(" ").add!("World").build()

        if result != "Hello World" {
            let error = 1 / 0  # Method chaining failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_with_function_fields() {
    let source = r#"
        struct Calculator {
            value: 0
            operation: null
        }

        fn add_ten(x) {
            return x + 10
        }

        let calc = Calculator { value: 5, operation: add_ten }
        let result = calc.operation(calc.value)

        if result != 15 {
            let error = 1 / 0  # Function field in struct failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_equality() {
    let source = r#"
        struct Point {
            x: 0
            y: 0
        }

        let p1 = Point { x: 10, y: 20 }
        let p2 = Point { x: 10, y: 20 }
        let p3 = Point { x: 5, y: 15 }

        # Structs should be compared by reference, not value
        if p1 == p2 {
            let error = 1 / 0  # Different struct instances shouldn't be equal
        }

        let p4 = p1
        if p1 != p4 {
            let error = 1 / 0  # Same struct reference should be equal
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_in_control_flow() {
    let source = r#"
        struct Score {
            value: 0

            fn add!(self, points) {
                self.value = self.value + points
            }

            fn is_winner(self) {
                return self.value >= 100
            }
        }

        let score = Score { value: 0 }
        let rounds = [20, 30, 15, 40]

        for points in rounds {
            score.add!(points)
            if score.is_winner() {
                break
            }
        }

        if score.value != 105 {
            let error = 1 / 0  # Struct in loop failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_as_function_parameter() {
    let source = r#"
        struct Vector {
            x: 0
            y: 0
        }

        fn add_vectors(v1, v2) {
            return Vector {
                x: v1.x + v2.x,
                y: v1.y + v2.y
            }
        }

        let a = Vector { x: 3, y: 4 }
        let b = Vector { x: 1, y: 2 }
        let c = add_vectors(a, b)

        if c.x != 4 || c.y != 6 {
            let error = 1 / 0  # Struct as function parameter failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_struct_with_list_field() {
    let source = r#"
        struct Database {
            records: []

            fn add_record!(self, record) {
                self.records.push(record)
            }

            fn count(self) {
                return self.records.len()
            }

            fn find_by_index(self, index) {
                if index >= 0 && index < self.records.len() {
                    return self.records[index]
                }
                return null
            }
        }

        let db = Database { records: [] }
        db.add_record!("First")
        db.add_record!("Second")
        db.add_record!("Third")

        if db.count() != 3 {
            let error = 1 / 0  # Struct with list field failed
        }

        if db.find_by_index(1) != "Second" {
            let error = 1 / 0  # Struct method accessing list failed
        }
    "#;

    run_test(source);
}
