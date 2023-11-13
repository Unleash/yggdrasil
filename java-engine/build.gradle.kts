plugins {
    // Apply the java-library plugin for API and implementation separation.
    `java-library`
    id("com.diffplug.spotless") version "6.0.4";
}

repositories {
    // Use Maven Central for resolving dependencies.
    mavenCentral()
}

dependencies {
    // Use JUnit Jupiter for testing.
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")

    // use Mockito
    testImplementation("org.mockito:mockito-core:5.2.0")

    implementation("net.java.dev.jna:jna:5.13.0")

    implementation("com.fasterxml.jackson.core:jackson-core:2.15.2")

    implementation("com.fasterxml.jackson.core:jackson-databind:2.15.1")

    implementation("com.fasterxml.jackson.datatype:jackson-datatype-jsr310:2.14.2")
}

spotless {
    java {
        // Choose a formatter, for example, Google Java Format
        googleJavaFormat("1.12.0").aosp()
        removeUnusedImports()
        importOrder()
    }
}

// Apply a specific Java toolchain to ease working on different environments.
java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(11))
    }
}

tasks.named<Test>("test") {
    // Use JUnit Platform for unit tests.
    useJUnitPlatform()
}
