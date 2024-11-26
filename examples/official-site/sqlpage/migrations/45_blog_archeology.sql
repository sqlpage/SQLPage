
INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'How archaeology is gradually entering the era of free software',
        'A team of french archaeologists is working on the first all-digital excavation site, using SQLPage',
        'skull',
        '2024-07-02',
        '
> This is the english translation of an article [originally published in French on linuxfr.org](https://linuxfr.org/news/comment-l-archeologie-entre-progressivement-dans-l-ere-du-logiciel-libre).
> It illustrates how SQLPage is used by non-developers 

# How archaeology is gradually entering the era of free software

Archaeology has, since its beginnings, focused on cataloging, structuring and archiving data from excavations. In the field, it has long relied on creating forms, manually collecting information on paper, and hand drawing, transcribed during study phases onto digital media. It is only recently that some archaeologists have launched the movement of "all-digital" excavation. I propose to tell here the story of the digitization of archaeology, which, as you will see, relies in part on free software.


# What is an excavation site?

French archaeology is divided into two main branches: preventive archaeology, which intervenes during construction projects, and programmed archaeology, conducted on sites chosen to address research issues. Supervised by the Regional Archaeological Services of the Ministry of Culture, these activities are carried out by different organizations: public and private operators for preventive archaeology, and associations, CNRS or universities for programmed archaeology. The latter often mobilizes volunteers, especially students, offering them complementary practical training.

For the archaeologist, excavation is a tool, not an end in itself. What the archaeologist seeks is information. In essence, it''s about understanding the history of a site, its evolution, its inhabitants through the elements they left behind, whether it''s the ruins of their habitats, their crafts or their burials. This is all the more important as excavation is a destructive act, since the archaeologist dismantles his subject of study as the excavation progresses.

To be exploited, archaeological information must be organized according to well-established principles. The first key concept is the sedimentary layer (*Stratigraphic Unit* - SU), which testifies to a human action or a natural phenomenon. The study of the arrangement of these layers reveals the chronology of the site, the succession of events that took place there. These layers can be grouped into archaeological *facts*: ditches, cellars, burials, are indeed groupings of layers that define a specific element. Finally, the objects found in these layers, or *artifacts*, are cataloged and identified by their layer of origin, providing crucial chronological and cultural indications.

![mastraits site](https://github.com/lovasoa/SQLpage/assets/552629/3dbdf81e-b9d3-4268-a8e3-99e568feb695)

*The excavation site of the Necropolis of Mastraits, in Noisy-le-Grand (93).*

The actions carried out by the archaeologist throughout the site are also recorded. Indeed, the archaeologist carries out surveys, digs trenches, but also takes many photos, or drawings of everything he discovers as the site progresses. The documentation produced can be plethoric, and cataloging is essential.

This descriptive information is complemented by **spatial information**, the plan of the uncovered remains being essential for the analysis and presentation of results. The study of this plan, associated with descriptive and chronological information, highlights the major evolutions of the site or specific details. Its realization is generally entrusted to a topographer in collaboration with archaeologists.

At the end of the field phase, a phase of analysis of the collected data is carried out. This so-called post-excavation phase allows for processing all the information collected, carrying out a complete description, conducting the studies necessary for understanding the site by calling on numerous specialists: ceramologists, anthropologists, archaeozoologists, lithicists, carpologists, anthracologists, paleometallurgy specialists, etc.

This post-excavation phase initially results in the production of an operation report, the most exhaustive account possible of the site and its evolution. These reports are submitted to the Ministry of Culture, which judges their quality. They are not intended to be widely disseminated, but are normally accessible to anyone who requests them from the concerned administration. They are an important working basis for the entire scientific community.

Based on this report, the publication of articles in specialized journals allows for presenting the results of the operation more widely, sometimes according to specific themes or issues.

# Practice of archaeology: example in preventive archaeology

The use of numerous paper listings is a constant. These listings allow keeping up-to-date records of data in the form of inventory tables of layers, facts, surveys, photos, etc. Specific recording sheets are also used in many specialties of archaeology, such as funerary anthropology.

In the field, the unearthed elements are still, for a very large majority, drawn by hand, on tracing or graph paper, whether it''s a plan of remains or the numerous stratigraphic section drawings. This of course requires significant time, especially in the case of complex remains.
The use of electronic tacheometers, then differential GPS, has made it possible to do without tape measures, or grid systems, when excavating sites. Topographers, specifically trained, then began to intervene on site for the realization of general plans.

The documentary collection obtained at the end of an excavation is particularly precious. These are the only elements that will allow reconstructing the history of the site, by crossing these data with the result of the studies carried out. The fear of the disappearance of this data, or its use by others due to a remarkable discovery, is a feeling often shared within the archaeological community. The archaeologist may feel like a custodian of this information, even expressing a feeling of possession that goes completely against the idea of shared and open science. The idea that opening up data is the best way to protect it is far from obvious.

![conservation sheet, illustrating manual coloring of found skeleton parts](https://github.com/lovasoa/SQLpage/assets/552629/ca9c0f99-a520-4f2b-9826-ae49a89f844b)
> *Conservation sheet, illustrating manual coloring of found skeleton parts*

![Example of a descriptive sheet of an archaeological layer](https://gitlab.com/projet-r-d-bddsarcheo/tutos/-/raw/main/illustrations_diverses/fiche_us.svg)
> *Example, among many others, of a blank descriptive sheet of an archaeological layer*

# The beginning of digitization

It is essentially after the field phase that digital tools have been tamed by archaeologists.

In post-excavation, paper documentation is still often a fundamental documentary basis for site analysis. The irruption of computing in the mid-80s led archaeologists to transcribe this data into digital form, to facilitate its analysis and presentation. Although the software has evolved, the process is practically the same today, with digitization of documentation in many formats.

Listings can be integrated into databases (most often proprietary such as MS Access, FileMaker or 4D) or spreadsheets. Many databases have been developed internally, locally, by archaeologists themselves. Only attributive, they have gradually networked and adapted to the medium, allowing consideration of use in the field, without this being widely deployed.

![Database](https://gitlab.com/projet-r-d-bddsarcheo/tutos/-/raw/main/illustrations_diverses/exemple_bdd_fmp.png) 
> *Example of a database at the turn of the 2000s*

All documentation drawn in the field is to be redrawn cleanly on digital media, in vector drawing software, very often Adobe Illustrator, sometimes Inkscape.
Plan data, surveyed by the topographer, is carried out under Autocad and was exported in .dxf or .dwg before being cleaned up under Adobe Illustrator, which is also the case for drawings made in the field.
The artifacts are entrusted to specialists who describe them, draw them, make an inventory, most often in spreadsheets. Their drawings are again scanned and cleaned up digitally.

In hindsight, we find that digital tools are mainly used as tools for cleaning up information collected in the field. Many spreadsheets are thus the strict transcription of paper tables used by archaeologists, to which some totals, averages or medians will be added. Drawings made on paper are traced in vectorization software for better readability and the scientific added values are ultimately quite limited.

This results in relatively disparate digital documentation, with the use of many proprietary tools, closed formats, and a very strong separation between spatial information and descriptive (or attributive) information.

The progressive use of databases has, however, allowed for agglomerating certain data and gathering and relating information. University work has also helped to feed reflection on the structuring of archaeological data and to train many archaeologists, allowing for the adoption of more virtuous practices.

# The all-digital movement

Until now, going fully digital in the archaeological context seemed relatively utopian. It took new technologies to appear, portable and simple-to-use supports to be put in place, networks to develop, and archaeologists to seize these new tools.

The Ramen collective (Archaeological Research in Digital Recording Modeling) was born from the exchanges and experiences of various archaeologists from the National Institute of Preventive Archaeological Research (Inrap) who grouped around the realization of [the programmed excavation of the medieval necropolis of Noisy-Le-Grand](https://archeonec.hypotheses.org/), excavation managed by the Necropolis Archaeology Association and entrusted to the scientific direction of Cyrille Le Forestier (Inrap). This programmed excavation allowed launching an experiment on the complete dematerialization of archaeological data based on photogrammetry, GIS, and a spatial database.

## General principle

While the topographer still intervenes for taking reference points, the detailed survey of remains is ensured, for this experiment, by the systematic implementation of photogrammetry. This method allows, by taking multiple photos of an object or scene, to create an accurate 3D model, and therefore exploitable a posteriori by the archaeologist in post-excavation. Photogrammetry constitutes in Noisy the only survey tool, purely and simply replacing drawing on paper. Indeed, from this 3D point cloud, it is possible to extract multiple 2D supports and add geometry or additional information to the database: burial contours, representation of the skeleton in situ, profiles, measurements, altitudes, etc.

![Photogrammetric survey of a burial](https://gitlab.com/projet-r-d-bddsarcheo/tutos/-/raw/main/illustrations_diverses/photogrammetrie3.png)
[*Photogrammetric survey of a burial*](https://sketchfab.com/3d-models/973-5d7513dd1dc941228d4a4b7b984c7af7)

Data recording is ensured by the use of a relational and spatial database whose interface is accessible in QGIS, but also via a web interface directly in the field, without going through paper inventories or listings. The web interface was created using [SQLPage](https://sql-page.com/), a web server that uses an SQL-based language for creating the graphical interface, without having to go through more complex programming languages classically used for creating web applications, such as PHP.

Of course, this approach also continues in the laboratory during the site analysis stage.

## Free software and formats

But abandoning paper support requires us to question the durability of the files and the data they contain.

Indeed, in a complete dematerialization process, the memory of the site is no longer contained on hundreds of handwritten sheets, but in digital files of which we do not know a priori if we will be able to preserve them in the long term. The impossibility of accessing this data with other software than those originally used during their creation is equivalent to their destruction. Only standard formats can address this issue, and they are particularly used by free software. For photogrammetry, the [`.ply`](https://en.wikipedia.org/wiki/PLY_(file_format)) and [`.obj`](https://en.wikipedia.org/wiki/Wavefront_.obj_file) formats, which are implemented in many software, free and proprietary, were chosen. For attributive and spatial data, it is recorded in free relational databases (Spatialite and Postgis), and easily exportable in `.sql`, which is a standardized format recognized by many databases.

Unfortunately, free software remains little used in our archaeological daily life, and proprietary software is often very well established. Free software still suffers today from preconceptions and a bad image within the archaeological community, which finds it more complicated, less pretty, less effective, etc.

However, free software has made a major incursion with the arrival of the free Geographic Information System (GIS) [QGIS](https://www.qgis.org/en/site/), which allowed installing a GIS on all the agents'' workstations of the institute and considering it as an analysis tool at the scale of an archaeological site. Through support and the implementation of an adequate training plan, many archaeologists have been trained in the use of the software within the Institute.

QGIS has truly revolutionized our practices by allowing immediate interrogation of attributive data by spatial data (what is this remains I see on the plan?) or, conversely, locating remains by their attributive data (where is burial 525?). However, it is still very common to have on one side the attributive data in spreadsheets or proprietary databases, and spatial data in QGIS, with the interrogation of both relying on joins.

Of course, QGIS also allows data analysis, the creation of thematic or chronological plans, essential supports for our reflections. We can, from these elements, create the numerous figures of the operation report, without going through vector drawing software, in plan as in section (vertical representation of stratigraphy). It allows normalizing figures through the use of styles, and, through the use of the Atlas tool, creating complete catalogs, provided that the data is rigorously structured.

![spatial analysis](https://gitlab.com/projet-r-d-bddsarcheo/tutos/-/raw/main/illustrations_diverses/ex_plan_analyse.png?ref_type=heads)
> *Example of spatial analysis in Qgis of ceramic waste distribution on a Gallic site*

In the context of the experiment on the Mastraits necropolis, while Qgis is indeed one of the pillars of the system, a few proprietary software are still used.

The processing software used for photogrammetry is proprietary. The ultimate goal is to be able to use free software, MicMac, developed by IGN, being a possible candidate. However, it still lacks a fully intuitive interface for archaeologists to appropriate the tool autonomously.

Similarly, the exciting latest developments of the Inkscape project should encourage us to turn more towards this software and systematically use .svg. The use of Scribus for DTP should also be seriously considered.

Free software and its undeniable advantages are thus slowly taking place, mainly via QGIS, in the production chain of our archaeological data. We can only hope that this place will grow. The path still seems long, but the way is free...

## Badass, spatial and attributive united

The development of the Archaeological Database of Attributive and Spatial Data (Badass) aimed to integrate, within a single database, the attributive information provided by archaeologists and the spatial information collected by the topographer. It even involves gathering, within dedicated tables, attributive and spatial information, thus ensuring data integrity.
Its principle is based on the functioning of the operational chain in archaeology, namely the identification and recording by the archaeologist of the uncovered remains, followed by the three-dimensional survey carried out by the topographer. The latter has, in the database, specific tables in which he can pour the geometry and minimal attributive data (number, type). Triggers then feed the tables filled by archaeologists with geometry, according to their identifier and type.

The database is thus the unique repository of attributive and spatial information throughout the operation, from field to post-excavation.

The format of the database was originally SpatiaLite. But the mass of documentation produced by the Mastraits necropolis led us to port it to PostGIS. Many archaeological operations, however, only require a small SpatiaLite base, which also allows the archaeologist to have control over their data file. Only a few large sites may need a PostgreSQL solution, otherwise used for the ARchaeological VIsualization CATalogue (Caviar) which is intended to host spatial and attributive data produced at the institute.

Naturally, Badass has been coupled with a QGIS project already offering default styles, but also some queries or views commonly used during an archaeological study. A QGIS extension has been developed by several students to allow automatic generation of the project and database.

Here''s the translation into idiomatic American English, keeping the original formatting:

## Entering Badass: The Bad''Mobil

The question of the system''s portability remained. QGIS is a resource-intensive software with an interface ill-suited for small screens, which are preferred for their portability in the field (phones and tablets).

Choosing to use a SpatiaLite or PostGIS database allowed us to consider a web interface from the start, which could then be used on any device. Initially, we considered developing in PHP/HTML/CSS with an Apache web server. However, this required having a web server and programming an entire interface. There were also some infrastructure questions to address: where to host it, how to finance it, and who would manage it all?

It was on LinuxFR that one of the members of the collective discovered [SQLPage](https://sql-page.com/). This open-source software, developed by [lovasoa](https://linuxfr.org/users/lovasoa), provides a very simple web server and allows for the creation of a [CRUD](https://en.wikipedia.org/wiki/CRUD) application with an interface that only requires SQL development.

SQLPage is based on an executable file which, when launched on a computer, turns it into a web server. A configuration file allows you to define the location of the database to be queried, among other things. For each web page of the interface, you write a `.sql` file to define the data to fetch or modify in the database, and the interface components to display it (tables, forms, graphs...). The interface is accessed through a web browser. If the computer is on a network, its IP address allows remote access, with an address like `http://192.168.1.5:8080`, for example. Using a VPN allows us to use the mobile phone network, eliminating the need for setting up a local network with routers, antennas, etc.

![principle](https://gitlab.com/projet-r-d-bddsarcheo/tutos/-/raw/main/illustrations_diverses/sqlpage_badass.svg)
*General operating principle*

Thus, the installation of the entire system is very simple and relies only on a file structure to be deployed on the server: the database, and a directory containing the SQLPage binary and the files making up the web pages.

By relying on the documentation (and occasionally asking questions to the software''s author), we were able to develop a very comprehensive interface on our own that meets our needs in the field. Named Bad''Mobil, the web interface provides access to all the attribute data recorded by archaeologists and now allows, thanks to the constant development of SQLPage, **to visualize spatial data**. Documentation produced during the excavation can also be consulted if the files (photos, scanned drawings, etc.) are placed in the right location in the file structure. The pages mainly consist of creation or modification forms, as well as tables listing already recorded elements. The visualization of geometry allows for spatial orientation in the field, particularly in complex excavation sites, and interaction with attribute data.

[![The BadMobil interface, with SQLPage](https://github.com/lovasoa/SQLpage/assets/552629/b421eebd-1d7a-446a-90d4-f360300453d5)](https://gitlab.com/projet-r-d-bddsarcheo/tutos/-/raw/main/illustrations_diverses/interface_badmobil.webp?ref_type=heads)
*The BadMobil interface, with SQLPage*

# Use Cases and Concrete Benefits

## First Experience at Les Mastraits

The excavation of the [Les Mastraits Necropolis](https://www.inrap.fr/la-necropole-alto-medievale-des-mastraits-noisy-le-grand-15374) was the test site for these developments. The significant amount of data collected, as well as its status as a planned excavation, allows for this kind of experimentation with much less impact than in a preventive excavation where deadlines are particularly tight.

The implementation of the SQLPage interface has allowed for the complete digitization of attribute recording and proves to be very efficient. This is a major change in our practices and will save us an enormous amount of time during data processing.

This also allows for centralizing information, working with multiple people simultaneously without waiting for traditional recording binders to become available, and guiding archaeologists through the recording process, avoiding omissions and errors. Thanks to a simplified interface, data entry can be done very intuitively without the need for extensive training.

The homogeneity of the entered data is thus better, and the possibilities for querying are much greater.

## Future Prospects

Following the development of Badass and Bad''mobil at the Les Mastraits necropolis, it seemed possible to consider its deployment in the context of preventive archaeology. While the question of the network infrastructure necessary for the operation of this solution may arise (need for stable electricity supply on remote sites in the countryside, availability of tablets, network coverage...), the benefits in terms of data homogeneity and ease of entry are very significant. A few preventive archaeology sites have thus been able to test the system, mostly on small-scale sites, benefiting from the support of collective members.

Future developments will likely focus on integrating new forms or new monitoring tools. Currently, Badass allows for collecting observations common to all archaeological sites, as well as anthropological observations due to its use within the Les Mastraits necropolis.
We could consider integrating the many specialties of archaeology, but it''s likely that we would end up with a huge machine that could be complex to maintain. We therefore remain cautious on this subject.

# Conclusion

Gradually, the use of digital tools has become widespread in archaeological professions. After the word processors and spreadsheets of the 90s (often on Mac), the first vectorized drawings digitized in Adobe Illustrator, and databases in Filemaker, Access, or 4D, digital tools are now able to be used throughout the entire data acquisition chain.

The contribution of open-source software and formats is major for this new step.

QGIS has fundamentally revolutionized archaeological practice by offering GIS access to the greatest number, allowing for the connection and manipulation of attribute and spatial data. It has paved the way for new developments and the integration of technologies previously little used by archaeology (notably the use of relational and spatial databases in SQL format).
SQLpage has allowed us to offer archaeologists a complete and simple interface to access a networked database. While its development requires certain knowledge of SQL and website functioning, its deployment and maintenance are quite manageable.
SQLPage addresses a real need in the field. For archaeologists, it simplifies their practice while responding to the growing complexity in the face of the documentary mass to be processed, and the increasing qualitative demands of deliverables.

The combination of QGIS, spatial and relational databases, and a web interface perfectly adapted to the field now fills the observed lack of an effective and reliable archaeological recording tool at the operation level. As such, Badass associated with Bad''Mobil fully meets the expectations of archaeologists who have experimented with them.

While open-source software has, in recent years, begun a timid breakthrough among many archaeological operators (some have fully adopted them), reluctance remains, whether from users or sometimes from the IT departments of public administrations, who may prefer to opt for an all-in-one service with technical support.

But the persistence of proprietary software usage is not without posing real problems regarding the sustainability of archaeological data, and archaeologists are just beginning to discover the issue. Their attachment to their data -- although it sometimes goes against the principle of open science -- should, however, encourage them to opt for formats whose durability appears certain, thereby guaranteeing access to this data in the future, regardless of the software or operating system used, if they don''t want their work to fall into oblivion...
        '
    );