// code snippet to tranform the roster into json

fn print_roster_json() {
    let lines = [
        "Toriees	DK	T	Furiees	DR	T",
        "Bensmonk	MO	T	Bensdh	DH	T	Benspriest	PR	R",
        "Câlyssa	SH	H	Càlyssa	PR	H	Calyssá	MO	H",
        "Foghammar	SH	H",
        "Twicepriest	PR	H",
        "Druuls	PA	H",
        "Totanká	DR	H	Mírwenn	PA	H",
        "Zlayèr	WL	R	Zorkón	SH	R",
        "Gulsvien	WL	R",
        "Dhaiva	WL	R",
        "Sneepsin	MA	R",
        "Räven	MA	R	Ravend	DR	R",
        "Óakenbow	HU	R",
        "Dermó	HU	R	Nachteule	DR	R	Nighthusky	MA	R",
        "Junezzhunter	HU	R	Junezz	DK	M",
        "Autlímit	DR	R",
        "Knottetwo	DR	R	Knottepriest	PR	R",
        "Jeadar	RO	M	Jeadwar	WA	M	Jeadsha	SH	M",
        "Möksy	WA	M",
        "Kiraddin	DH	M	Kiradk	DK	M",
        "Schlomey	SH	M",
        "Donjuanó	MO	M	Donwizard	MA	R",
        "Rêsu	MO	M	Resudh	DH	M",
        "Benevølent	DK	M",
        "Riverice	DK	M	Axlvg	RO	M"];

    let mut i = 0;
    for l in lines.iter() {
        println!("\"{}\": {{", i);

        let mut words = l.split_ascii_whitespace().peekable();
        let mut first = true;
        while true {
            match words.next() {
                Some(w) => println!("\t\"{}\": {{", w),
                None => break
            }
            println!("\t\t\"class\": \"{}\",", words.next().unwrap());
            println!("\t\t\"role\": \"{}\",", words.next().unwrap());
            println!("\t\t\"main\": {}", first);
            println!("\t}}{}", 
                match words.peek() {
                    Some(_) => ",",
                    None => "",
                });
            first = false;
        }
        i+=1;

        println!("}},");
    }
}