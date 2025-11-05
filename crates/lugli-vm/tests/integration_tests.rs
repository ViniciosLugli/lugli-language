mod helpers;
use helpers::run_test;

#[test]
fn test_todo_list_app() {
    let source = r#"
        # Simple todo list application
        struct TodoItem {
            text: ""
            done: false

            fn complete!(self) {
                self.done = true
            }

            fn toggle!(self) {
                self.done = !self.done
            }
        }

        struct TodoList {
            items: []

            fn add!(self, text) {
                let item = TodoItem { text: text, done: false }
                self.items.push(item)
            }

            fn complete_item!(self, index) {
                if index >= 0 && index < self.items.len() {
                    self.items[index].complete!()
                }
            }

            fn get_pending(self) {
                let pending = []
                for item in self.items {
                    if !item.done {
                        pending.push(item.text)
                    }
                }
                return pending
            }

            fn count_done(self) {
                let count = 0
                for item in self.items {
                    if item.done {
                        count = count + 1
                    }
                }
                return count
            }
        }

        let todos = TodoList { items: [] }
        todos.add!("Write tests")
        todos.add!("Fix bugs")
        todos.add!("Document code")

            if todos.items.len() != 3 {
                let error = 1 / 0  # Add todo items failed
            }


        todos.complete_item!(0)
        todos.complete_item!(2)

            if todos.count_done() != 2 {
                let error = 1 / 0  # Complete items failed
            }


        let pending = todos.get_pending()
        if pending.len() != 1 {
            let error = 1 / 0  # Get pending failed
        }

        if pending[0] != "Fix bugs" {
            let error = 1 / 0  # Wrong pending item
        }
    "#;

    run_test(source);
}

#[test]
fn test_calculator_with_history() {
    let source = r#"
        struct Calculator {
            result: 0
            history: []

            fn add!(self, value) {
                self.result = self.result + value
                let entry = "add " + str(value)
                self.history.push(entry)
                return self
            }

            fn multiply!(self, value) {
                self.result = self.result * value
                let entry = "multiply " + str(value)
                self.history.push(entry)
                return self
            }

            fn clear!(self) {
                self.result = 0
                self.history = []
                return self
            }

            fn get_result(self) {
                return self.result
            }
        }

        let calc = Calculator { result: 0, history: [] }
        calc.add!(10)
        calc.multiply!(3)
        calc.add!(5)

        if calc.get_result() != 35 {
            let error = 1 / 0  # Calculator operations failed
        }

        if calc.history.len() != 3 {
            let error = 1 / 0  # History tracking failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_student_grades_system() {
    let source = r#"
        struct Student {
            name: ""
            grades: []

            fn add_grade!(self, grade) {
                self.grades.push(grade)
            }

            fn average(self) {
                if self.grades.len() == 0 {
                    return 0
                }
                let sum = 0
                for grade in self.grades {
                    sum = sum + grade
                }
                return sum / self.grades.len()
            }

            fn is_passing(self) {
                return self.average() >= 60
            }
        }

        struct Classroom {
            students: []

            fn add_student!(self, student) {
                self.students.push(student)
            }

            fn get_passing_students(self) {
                let passing = []
                for student in self.students {
                    if student.is_passing() {
                        passing.push(student.name)
                    }
                }
                return passing
            }

            fn class_average(self) {
                if self.students.len() == 0 {
                    return 0
                }
                let total = 0
                for student in self.students {
                    total = total + student.average()
                }
                return total / self.students.len()
            }
        }

        let classroom = Classroom { students: [] }

        let alice = Student { name: "Alice", grades: [] }
        alice.add_grade!(85)
        alice.add_grade!(92)
        alice.add_grade!(88)

        let bob = Student { name: "Bob", grades: [] }
        bob.add_grade!(55)
        bob.add_grade!(62)
        bob.add_grade!(58)

        let charlie = Student { name: "Charlie", grades: [] }
        charlie.add_grade!(95)
        charlie.add_grade!(98)
        charlie.add_grade!(92)

        classroom.add_student!(alice)
        classroom.add_student!(bob)
        classroom.add_student!(charlie)

        let passing = classroom.get_passing_students()
        if passing.len() != 2 {
            let error = 1 / 0  # Passing students count wrong
        }

        if passing[0] != "Alice" || passing[1] != "Charlie" {
            let error = 1 / 0  # Wrong passing students
        }

        let avg = classroom.class_average()
        if avg < 75 || avg > 85 {
            let error = 1 / 0  # Class average calculation wrong
        }
    "#;

    run_test(source);
}

#[test]
fn test_recursive_fibonacci() {
    let source = r#"
        fn fibonacci(n) {
            if n <= 1 {
                return n
            }
            return fibonacci(n - 1) + fibonacci(n - 2)
        }

        if fibonacci(0) != 0 {
            let error = 1 / 0  # fib(0) failed
        }

        if fibonacci(1) != 1 {
            let error = 1 / 0  # fib(1) failed
        }

        if fibonacci(5) != 5 {
            let error = 1 / 0  # fib(5) failed
        }

        if fibonacci(10) != 55 {
            let error = 1 / 0  # fib(10) failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_bubble_sort() {
    let source = r#"
        fn bubble_sort(arr) {
            let n = arr.len()
            for i in range(n) {
                for j in range(n - i - 1) {
                    if arr[j] > arr[j + 1] {
                        let temp = arr[j]
                        arr[j] = arr[j + 1]
                        arr[j + 1] = temp
                    }
                }
            }
        }

        let numbers = [64, 34, 25, 12, 22, 11, 90]
        bubble_sort(numbers)

        # Check if sorted
        for i in range(numbers.len() - 1) {
            if numbers[i] > numbers[i + 1] {
                let error = 1 / 0  # Array not sorted correctly
            }
        }

        if numbers[0] != 11 || numbers[6] != 90 {
            let error = 1 / 0  # Sort endpoints wrong
        }
    "#;

    run_test(source);
}

#[test]
fn test_event_system() {
    let source = r#"
        struct Event {
            name: ""
            data: null

            fn init!(self, name, data) {
                self.name = name
                self.data = data
            }
        }

        struct EventBus {
            handlers: {}

            fn on!(self, event_name, handler) {
                if !self.handlers.contains(event_name) {
                    self.handlers[event_name] = []
                }
                self.handlers[event_name].push(handler)
            }

            fn emit!(self, event_name, data) {
                if !self.handlers.contains(event_name) {
                    return
                }

                let handlers = self.handlers[event_name]
                for handler in handlers {
                    handler(data)
                }
            }
        }

        let bus = EventBus { handlers: {} }
        let results = []

        fn handler1(data) {
            results.push("handler1: " + data)
        }

        fn handler2(data) {
            results.push("handler2: " + data)
        }

        bus.on!("test", handler1)
        bus.on!("test", handler2)
        bus.emit!("test", "hello")

        if results.len() != 2 {
            let error = 1 / 0  # Event handlers not called
        }
    "#;

    run_test(source);
}

#[test]
fn test_matrix_operations() {
    let source = r#"
        struct Matrix {
            data: []
            rows: 0
            cols: 0

            fn get(self, row, col) {
                let index = row * self.cols + col
                return self.data[index]
            }

            fn set!(self, row, col, value) {
                let index = row * self.cols + col
                self.data[index] = value
            }

            fn transpose(self) {
                let result = Matrix {
                    data: [],
                    rows: self.cols,
                    cols: self.rows
                }

                # Initialize result data
                for i in range(self.cols * self.rows) {
                    result.data.push(0)
                }

                for i in range(self.rows) {
                    for j in range(self.cols) {
                        result.set!(j, i, self.get(i, j))
                    }
                }

                return result
            }
        }

        let m = Matrix { data: [1, 2, 3, 4, 5, 6], rows: 2, cols: 3 }

        if m.get(0, 0) != 1 {
            let error = 1 / 0  # Matrix get(0,0) failed
        }

        if m.get(1, 2) != 6 {
            let error = 1 / 0  # Matrix get(1,2) failed
        }

        let t = m.transpose()

        if t.rows != 3 || t.cols != 2 {
            let error = 1 / 0  # Transpose dimensions wrong
        }

        if t.get(0, 0) != 1 || t.get(2, 1) != 6 {
            let error = 1 / 0  # Transpose values wrong
        }
    "#;

    run_test(source);
}

#[test]
fn test_closure_counter() {
    let source = r#"
        fn make_counter(start) {
            let count = start

            fn increment() {
                count = count + 1
                return count
            }

            return increment
        }

        let counter1 = make_counter(0)
        let counter2 = make_counter(100)

        if counter1() != 1 {
            let error = 1 / 0  # Counter1 first call failed
        }

        if counter1() != 2 {
            let error = 1 / 0  # Counter1 second call failed
        }

        if counter2() != 101 {
            let error = 1 / 0  # Counter2 first call failed
        }

        if counter1() != 3 {
            let error = 1 / 0  # Counter1 third call failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_data_pipeline() {
    let source = r#"
        let data = [
            {"name": "Alice", "age": 30, "city": "NY"},
            {"name": "Bob", "age": 25, "city": "LA"},
            {"name": "Charlie", "age": 35, "city": "NY"},
            {"name": "Diana", "age": 28, "city": "LA"},
            {"name": "Eve", "age": 32, "city": "NY"}
        ]

        # Filter by city
        let ny_residents = []
        for person in data {
            if person["city"] == "NY" {
                ny_residents.push(person)
            }
        }

        if ny_residents.len() != 3 {
            let error = 1 / 0  # Filter by city failed
        }


        # Get ages of NY residents
        let ages = []
        for person in ny_residents {
            ages.push(person["age"])
        }

        # Calculate average age
        let sum = 0
        for age in ages {
            sum = sum + age
        }
        let avg = sum / ages.len()

        if avg < 30 || avg > 33 {
            let error = 1 / 0  # Average age calculation failed
        }

        # Find oldest person
        let oldest = data[0]
        for person in data {
            if person["age"] > oldest["age"] {
                oldest = person
            }
        }

        if oldest["name"] != "Charlie" {
            let error = 1 / 0  # Finding oldest person failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_mini_game_logic() {
    let source = r#"
        struct Player {
            name: ""
            health: 100
            score: 0

            fn take_damage!(self, amount) {
                self.health = self.health - amount
                if self.health < 0 {
                    self.health = 0
                }
            }

            fn heal!(self, amount) {
                self.health = self.health + amount
                if self.health > 100 {
                    self.health = 100
                }
            }

            fn is_alive(self) {
                return self.health > 0
            }

            fn add_score!(self, points) {
                self.score = self.score + points
            }
        }

        struct Game {
            players: []
            round: 0

            fn add_player!(self, player) {
                self.players.push(player)
            }

            fn simulate_round!(self) {
                self.round = self.round + 1

                for player in self.players {
                    if player.is_alive() {
                        # Simulate random damage (using fixed values for test)
                        if self.round % 2 == 0 {
                            player.take_damage!(15)
                        } else {
                            player.heal!(10)
                        }

                        if player.is_alive() {
                            player.add_score!(10)
                        }
                    }
                }
            }

            fn get_winner(self) {
                let winner = null
                let max_score = -1

                for player in self.players {
                    if player.score > max_score {
                        max_score = player.score
                        winner = player
                    }
                }

                return winner
            }
        }

        let game = Game { players: [], round: 0 }

        let p1 = Player { name: "Alice", health: 100, score: 0 }
        let p2 = Player { name: "Bob", health: 100, score: 0 }

        game.add_player!(p1)
        game.add_player!(p2)

        # Simulate 5 rounds
        for i in [1, 2, 3, 4, 5] {
            game.simulate_round!()
        }

        if game.round != 5 {
            let error = 1 / 0  # Round counter failed
        }

        let winner = game.get_winner()
        if winner == null {
            let error = 1 / 0  # No winner found
        }

        if winner.score == 0 {
            let error = 1 / 0  # Winner has no score
        }
    "#;

    run_test(source);
}
