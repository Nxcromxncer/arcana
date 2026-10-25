? FizzBuzz: loops, modulo, and the strict boolean rules
spell main() {
    for n in range(1, 16) {
        if n % 15 == 0 {
            cast "FizzBuzz"
        } else if n % 3 == 0 {
            cast "Fizz"
        } else if n % 5 == 0 {
            cast "Buzz"
        } else {
            cast n
        }
    }
}
