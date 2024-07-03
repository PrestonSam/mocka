There are some key points worth raising with regards to the way that mockadoc should work.

The first point is that mockadoc files actually generate two different outputs, depending on what you want. The first thing that they generate is a "metadata" document (encoded as JSON). This is because mockadoc can be used to express invariants, such as the SQL type that's associated with a given column.

The other output is of course the CSV / TSV files that use the generators imported from mockagen files. For this, one of the columns in the mockadoc table is chosen to represent the headings for the output file, while the generator column is used for the rows.

Something worth noting here is that the 'metadata' output generates a list of anonymous objects and follows the same shape as the mockadoc file, while the TSV files only use given properties and transpose the document to have the keys at the top and the generators producing all subsequent rows.


I want to change the spec to support all this, but I'm not certain on how the new spec will look.

One option would be as follows:


```md
### Channel

|_Template name_|Internal name|SQL Type|`GENERATOR`|
|---|---|---|---|
|ISO_TIMESTAMP|UnixTimestamp|string|`unix-timestamp`|
|EVENT_TYPE|EventType|string|`channel-event-type`|
|REGION|ActorRegion|string|`region`|
```

Where _italics_ defines the key used in data generation and the generator (and metadata) tags are distinguished using backticks.


Another approach would be to dictate the key on a separate line as follows:

```md
### Channel

KEY: Template name

|Template name|Internal name|SQL Type|`GENERATOR`|
|---|---|---|---|
|ISO_TIMESTAMP|UnixTimestamp|string|`unix-timestamp`|
|EVENT_TYPE|EventType|string|`channel-event-type`|
|REGION|ActorRegion|string|`region`|
```

...Nah I don't much like that syntax. I guess we'll keep the italics syntax, then.
