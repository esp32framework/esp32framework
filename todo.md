- Siempre se inicializa el adcDriver, ver de solo inicializarloc cuando el usuario lo necesita
- Cuando el cliente declara un AnalogIn, en la firma de la funcion (si quiere envarlo a una funcion para algo) tiene que aclarar el <atten_t> y queda feo 

- sharable timers
- multi iterrupt timers

- En el get_temperature del DS3231 lo leido esta en complemento a 2, entonces hay que aplicarle eso para conseguir el decimal. Lo que no se es si tambien esos numeros en complemento a 2 tambien estaban con BCD. Por ahora solo se le saca el complemento a 2. Habria quee leer la docu para ver si no vienen en bcd tambien
- En la aprte del trait READER de i2c se devuelve un hashmap<String, String>. Lo que se puede hacer es definir un type "Clave" y un type "Valor" y que pase a devolver un hashmap<Clave, Valor>.
