The new [release v0.1.0](https://gitlab.com/uklotzde/aoide-rs/-/releases) includes a huge refactoring according to [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) principles. I have split the code internally into various sub-projects.

The new, backward incompatible JSON schema is optimized for space efficiency and reduces the size of the resulting database file by 20-25%. Request/response size should be reduced by about 30-40%.

Killer features are (still) *faceted tags* (facet + label + score) in combination with *prepared queries*.

The following properties are available for filtering in prepared queries:

*Strings*
* mediaUri (URL-decoded, i.e. may contain whitespace and reserved characters)
* mediaType
* trackTitle
* trackArtist
* trackComposer
* albumTitle
* albumArtist
*Numbers*
* audioBitRate
* audioChannelCount
* audioDuration
* audioLoudness
* audioSampleRate
* trackNumber
* trackTotal
* discNumber
* discTotal
* releaseYear
* musicTempo
* musicKey
*Tags*
* facet ("genre", "energy", ...symbolic strings, all lowercase, no whitespaces allowed)
* label
* score [0.0..1.0]

Query results can be sorted by any combination of:

* inCollectionSince
* lastRevisionedAt
* trackTitle
* trackArtist
* trackNumber
* trackTotal
* discNumber
* discTotal
* albumTitle
* albumArtist
* releaseYear
* musicTempo

The branch [dev_aoide](https://github.com/uklotzde/mixxx/tree/dev_aoide) contains a prototypical integration for Mixxx. Copy the resulting executable (~6.4 MB, Linux x86_64, statically linked) into your Mixxx settings folder and it will be started as a sub-process. Example prepared queries are included as a [JSON file](https://github.com/uklotzde/mixxx/blob/dev_aoide/res/aoide/example_prepared_queries.json) and must be loaded manually into the *aoide* feature in the side pane.


* *core*: A functional *core* with minimal dependencies that contains all the domain entities and their validation logic
* *core-serde*: A serialization layer around the *core* with bidirectional transformations for the domain entities, tuned for a space efficient JSON representation
* *repo*: A layer that defines interfaces with accompanying utilities to manage stateful, persistent storage for the domain entities in *core*
* *repo-sqlite*: A concrete implementation of the repository layer using SQLite as the backend
* ...the remaining code of the web service
