There are some key points worth raising with regards to the way that mockadoc should work.

The first point is that mockadoc files actually generate two different outputs, depending on what you want. The first thing that they generate is a "metadata" document (encoded as JSON). This is because mockadoc can be used to express invariants, such as the SQL type that's associated with a given column.

The other output is of course the CSV / TSV files that use the generators imported from mockagen files. For this, one of the columns in the mockadoc table is chosen to represent the headings for the output file, while the generator column is used for the rows.

Something worth noting here is that the 'metadata' output generates a list of anonymous objects and follows the same shape as the mockadoc file, while the TSV files only use given properties and transpose the document to have the keys at the top and the generators producing all subsequent rows.


I want to change the spec to support all this, but I'm not certain on how the new spec will look.

For this I will use the following syntax:

```md
### Channel

|_Template name_|Internal name|SQL Type|`GENERATOR`|
|---|---|---|---|
|ISO_TIMESTAMP|UnixTimestamp|string|`unix-timestamp`|
|EVENT_TYPE|EventType|string|`channel-event-type`|
|REGION|ActorRegion|string|`region`|
```

Where _italics_ defines the key used in data generation and the generator (and metadata) tags are distinguished using backticks.



Nah let's separate the schema from the outputs and make everything both generic and explicit - like so

```md
# Channel

## Schema
|Template name|Internal name|SQL Type|`Generator`|
|---|---|---|---|
|ISO_TIMESTAMP|UnixTimestamp|string|`unix-timestamp`|
|EVENT_TYPE|EventType|string|`channel-event-type`|
|REGION|ActorRegion|string|`region`|

## Outputs
 - ### Tabular
    - Formats
        - TSV
        - CSV
    - Column names
        - Template name
    - Row values
        - Generator
 - ### Document
   - Formats
     - JSON
   - Members
     - Template name
     - Internal name
     - SQL Type

```

I'm not _quite_ convinced by this syntax, but I think it's much closer to a good design than before.

I think it's still worth putting special syntax for the headings of generator columns, so that it's a syntax error if you don't have a homogenous column type.
