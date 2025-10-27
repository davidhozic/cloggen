# CLOGGEN
Generator študentskih mnenj (za habilitacijo).

## Namestitev pisave (font)
Generiran dokument uporablja pisavo *Roboto*. V primeru, da pisava na sistemu ni nameščena, se dokument ne bo generiral.
Za namestitev uporabi datoteke v mapi ``mnenja-template/fonts/Roboto``. Namesti vse datoteke.

## Prevajanje Cloggen

Cloggen se zanaša na LaTeX prevajalnik imenovan [Tectonic](https://tectonic-typesetting.github.io/book/latest/index.html).
Tectonic se zanaša na določene sistemske knjižnice, ki jih je potrebno predhodno namestiti.

Neodvisno od operacijskega sistema je predhodno potrebno nastaviti C++ standard na C++17 (`CXXFLAGS=-std=c++17`).

### Linux
Prevajanje na Linuxu je enostavno. Namestiti je potrebno pakete preko upravljalnika paketov:

    sudo apt-get install libfontconfig1-dev libgraphite2-dev libharfbuzz-dev libicu-dev libssl-dev zlib1g-dev

Cloggen se potem lahko prevede z:

    cargo build --release

in namesti z:

    cargo install --path .

### Windows
Prevajanje na Windows sistemih je malo bolj tečno.
Najlažja pot:

- namesti Visual Studio s CMake
- namesti vcpkg:

    - ``git clone https://github.com/microsoft/vcpkg.git``
    - ``cd vcpkg; .\bootstrap-vcpkg.bat``
    - ``./vcpkg install fontconfig freetype "harfbuzz[graphite2]" icu --triplet x64-windows-static-release``

- pripravi okolje (uporaba **Git Bash**):

    - pojdi v vcpkg mapo repozitorija in kopiraj pot (npr. s ``pwd``)
    - ``export VCPKG_ROOT="polna pot do vcpkg mape tukaj"``
    - ``export RUSTFLAGS='-Ctarget-feature=+crt-static'``
    - ``export VCPKGRS_TRIPLET='x64-windows-static-release'``
    - ``export TECTONIC_DEP_BACKEND=vcpkg``


Cloggen se potem lahko prevede z:

    cargo build --release

in namesti z:

    cargo install --path .

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


