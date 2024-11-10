# CLOGGEN
Generator študentskih mnenj (za habilitacijo).

## Namestitev pisave (font)
Generiran dokument uporablja pisavo *Roboto*. V primeru, da pisava na sistemu ni nameščena, se dokument ne bo generiral.
Za namestitev uporabi datoteke v mapi ``data/fonts/Roboto``. Namesti vse datoteke.

## Generiranje dokumentov

Za generiranje dokumenta uporabi ukaz:

    cloggen create <CSV DATOTEKA STUDIS ANKET> <JSON NABOR ODZIVOV> <TEX DOKUMENT> -f <FORMAT> -o <IZHODNA POT>

- ``<CSV DATOTEKA STUDIS ANKET>`` predstavlja izvoženo CSV datoteko z ocenami kandidata za posamezno vprašanje STUDIS anket
- ``<JSON NABOR ODZIVOV>`` predstavlja JSON datoteko, ki definira odgovore za posamezno mejo ocene v formatu:
    ```json
        {
        "Vprašanje": {
            "Gledano v celoti, je delo izvajalca/ke kakovostno.": {
                "1": ["Odziv 1", "Odziv 2", ...],
                "1.5": ["Odziv 1", "Odziv 2", ...],
                ...
                "4": ["Odziv 1", "Odziv 2", ...],
                "4.5": ["Kandidat ima super ocene (povprečje {MEAN} $\\pm$ {STD}).", "Odziv 2", ...],
            }
        }
    }
    ```

    Odzivi so razporejeni po večih številkah. Številke so minimalna meja povprečne ocene pri posameznem vprašanju, ki
    jo mora kandidat imeti, zato da dobi enega izmed pripadajočih odzivov.
    
    Odziv bo izbran iz možnih odzivov, ki pripadajo prvi manjši oceni od povprečne ocene kandidata. Na primer, če ima
    kandidat pri vprašanju *Gledano v celoti, je delo izvajalca/ke kakovostno.* povprečno oceno 4.3, bo ob uporabi
    zgornjega JSON primera odziv izbran iz odzivov, ki pripadajo oceni 4.0 (``"4": ["Odziv 1", "Odziv 2", ...]``)

    V odziv se lahko dinamično vključi tudi **povprečje** in **standardni odklon**, kot prikazuje zgornjni JSON primer:
    ``"4.5": ["Kandidat ima super ocene (povprečje {MEAN} $\\pm$ {STD}).", ...]``. Tu bo ``{MEAN}`` z povprečno oceno za 
    pripadajoče vprašanje, ``{STD}`` pa s standardnim odklonom za pripadajoče vprašanje.

- ``<TEX DOKUMENT>`` predstavlja glavni LaTeX dokument (datoteko),
    ki bo uporabljen za generacijo izhodnega mnenja v PDF obliki.
    Dokument mora vsebovati ``{AUTO_GEN}`` tekst, ki predstavlja lokacijo
    vstavitve odzivov/odgovorov, generiranih iz zgornje JSON datoteke odzivov.

- ``<FORMAT>`` predstavlja izhodni format. Privzeta vrednost je ``pdf`` (izhod bo .pdf datoteka),
    lahko pa se izbere tudi ``latex`` (izhod bo .tex latex datoteka).
- ``<IZHODNA POT>`` predstavlja pot, kamor bo shranjen generiran dokument.
    Privzeto je ta vrednost enaka ``output_<TEX DOKUMENT>.<tex/pdf>``.

## Združevanje STUDIS anket
Cloggen omogoča združevanje večih STUDIS CSV datotek v eno skupno datoteko.
Združijo se le povprečne ocene posameznih datotekek, tako, da se povprečijo.
Standardni odklon je na novo izračunan iz povprečij datotek.

Na primer, če imamo dve datoteki s povprečji ocen 3.2 in 5.0, potem bo novo povprečje enako 4.1, standarndi odklon pa
bo enak 0.90.

### Uporaba

    cloggen merge <csv1> <csv2> ...

Izhodna pot združene datoteke je privzeto ``./merged.csv``. Za lastno pot uporabi ``-o <IZHODNA POT>``
