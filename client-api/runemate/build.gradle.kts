plugins {
    kotlin("jvm") version "1.6.10"
}

group = "de.torsm.osrs-nav"
version = "0.1.0"

repositories {
    mavenCentral()
    maven("https://gitlab.com/api/v4/projects/10471880/packages/maven")
    maven("https://gitlab.com/api/v4/projects/32972353/packages/maven")
}

dependencies {
    implementation(kotlin("stdlib"))
    implementation("com.runemate:runemate-client:3.1.5.0:all")
    implementation("com.runemate:runemate-game-api:1.2.4")
    implementation(files("../core/src"))
}

tasks.register<Copy>("buildRuneMateTestBot") {
    from(tasks.named("compileTestKotlin"))
    from(tasks.named("processTestResources"))
    into(System.getProperty("user.home") + "/RuneMate/bots/")
}
