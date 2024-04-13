# The Mocka Programming Language

## What is Mocka?

Mocka is a pair of programming languages that can be used to generate artificial data.

The target audience of these languages is users that need to generate data locally, using complex and interrelated rules, but lack the experience in programming necessary for data generation libraries such as [FakerJS](https://fakerjs.dev/).

In a commercial environment, it may be dubious to generate data using an online tool such as [Fakery](https://fakery.dev/), as properly configuring such a tool may involve divulging protected information in order to generate convincing mock data.


## How to use Mocka

There are two languages that make up Mocka:

### Mockadoc
Used for describing the columns for a prospective document that could be generated i.e. a CSV file. Mockadoc files contain column headings, types, generators (imported from mockagen files) and information on how the columns relate to one another. Columns marked as "personal" are linked with an "identity" which may reoccur in multiple rows. This is to make analytics possible on the data.


#### Example code
```
EVENT "Transaction" TYPEDEF "event-types.mkg"
 - "Timestamp" UnixTimestamp string unix-timestamp AS PRIMARY TIMESTAMP
 - "Name" ActorName string full-name 
 - "Age" ActorAge long age AS PERSONAL
 - "Country" ActorCountry long country AS PERSONAL
 - "Region" ActorRegion string region AS PERSONAL

```

### Mockagen
Used for describing generators. Each generator defines the rules for how to generate a specific type of datapoint. Generators can reference one another to impose conditions on what value they can be - for example the generator for a person's country would influence the town they might be from.

#### Example code
```
INCLUDE "some-file.mkg", "some-other-file.mkg"


DEF unix-timestamp = timestamp/date 2023-05-08 2023-07-07

DEF age = integer 18 90

DEF country, currency-code 
    = 2% "United Kingdom"
        = "GBP"
    = 3% "China"
        = "CNY"
    = 14% "Europe"
        = "EUR"
    = 10% "USA"
        = "USD"
    = "Japan"
        = "JPY"

USING country DEF region
    ? "United Kingdom"
        = ONEOF
        | 17% "London"
        | 10% "Manchester"
        | 5% "Liverpoole"
        | 11% "Leeds"
        | 13% "Birmingham"
        | 3% "Cardiff"
        | 6% "Reading"
        | 9% "Bristol"
        | 2% "Oxford"
        | 4% "Glasgow"
        | 8.2% "Southampton"
        | "Cambridge"
        | "Edinburgh"
    ? any
        = "Unknown"

DEF first-name
    = ONEOF
    | "Tom"
    | "Dick"
    | "Harry"

DEF surname
    = ONEOF
    | "Smith"
    | "Baker"
    | "Carpenter"

DEF full-name = join surname "," first-name

```

## Project Status

### Mockagen
- [x] Parser (inc. tokeniser)
- [x] AST builder
- [ ] Evaluation
    - [ ] Design interface
    - [ ] Implement evaluator

### Mockadoc
- [ ] Parser (inc. tokeniser)
- [ ] AST builder
- [ ] Evaluation

