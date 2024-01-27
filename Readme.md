# OSMeili

A (reverse-)geocoder backed by [meilisearch](https://www.meilisearch.com/)

## What

This is a geocoder similar to the likes of [Photon](https://github.com/komoot) or [mimirsbrunn](https://github.com/Qwant/mimirsbrunn)
the difference between them is quite significant for a few reasons:

- It uses [meilisearch](https://www.meilisearch.com/) as it's backing store instead of Elasticsearch<br>
  This makes it both more future-proof in terms of licensing and more performant

- It's very simple and only does basic geocoding and reverse geocoding<br>
  At the moment, it only considers OSM nodes, not streets (ways) or administrative boundaries.
  This was done to reduce complexity in the hopes that quality will be good enough.<br>
  Should the quality be lacking, features like boundaries will gradually be added.

- It's relatively fast and lightweight<br>
  Importing the extract of Germany (4Gb) takes ~250Mb of memory and roughly 5 minutes.[^1]<br>
  Planet scale imports take ~500Mb of memory and ~2 hours[^2]

- It does not do automatic updates yet. This will be added in a future version
- It can be easily tweaked to only import what you really need<br>
  Since different people have different needs, you can filter for specific OSM tags you want to include.
  By default, [all addr keys](https://wiki.openstreetmap.org/wiki/Key:addr:*) are considered

- You can bring your own parser or use the default (tbd)<br>
  Thanks to being tweak-able, you can choose whether you want to build your own client and interact directly
  with the meilisearch API or use the default wrapper using the defaults from 6.

- Native container support (tbd)<br>
  Simply use the image in your spec and get everything running in minutes

## Why

Nominatim is great, but it doesn't do fuzzy search. Photon being Java + Elastic turned out to be slow
in multiple test cases, plus it uses an old version of Elastic for [licensing reasons](https://github.com/komoot/photon/issues/285).

Mimirsbrunn looked very promising until Hove closed-sourced it. So far none of the forks seem to have continued
the development effort, so it is pretty much dead. Plus, it's also using Elastic.

I took this as an opportunity to learn more about Rust, meilisearch and OSM, hoping it could one day become
a viable alternative (truth be told, it's still very far from it)

## Future
A non-exhaustive list of things that are currently missing

- [ ] Importing from URL or local file (planned)
- [ ] Updating entries based on set frequency (planned)
- [ ] Default API client handling meili interaction (planned)
- [ ] Status page (planned)
- [ ] Docker/Kubernetes packaging & deployment instructions (planned)
- [ ] Offer admin boundaries and streets (contributions welcome)

## Special thanks
- Magnus Kulke for his initial work on [osm-pbf2json](https://github.com/mkulke/osm-pbf2json) which this project is based on[^3]
- Giora Kosoi from navigatorsguild for [osm-io](https://github.com/navigatorsguild/osm-io)
  they also have a similar project trying to replace Nominatim, [check it out](https://github.com/navigatorsguild/osm-admin)

## License

This work is dual-licensed under MIT and EUPL 1.2.
You must accept with both of them if you use this work.

`SPDX-License-Identifier: MIT AND EUPL-1.2`

[^1]: "Measured with an M1 Macbook Pro running Linux, let me know of your experience"
[^2]: The import itself takes around 50% of the time.
After the import is done, indexes are tweaked for better search result quality. If you do not need this, you can disable it.
[^3]: All code created by original authors is licensed under MIT. See LICENSE.MIT for details