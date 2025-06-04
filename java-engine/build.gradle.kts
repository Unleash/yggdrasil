plugins {
    `java-library`
    `maven-publish`
    signing
    id ("com.diffplug.spotless").version("6.23.2")
    id("io.github.gradle-nexus.publish-plugin").version("2.0.0")
    id("pl.allegro.tech.build.axion-release").version("1.16.0")
    id("tech.yanand.maven-central-publish").version("1.3.0")
    id("me.champeau.jmh").version("0.7.2")
    id("at.released.wasm2class.plugin").version("0.3")
}

wasm2class {
    modules {
        create("Yggdrasil") {
            wasm = file("../target/wasm32-unknown-unknown/release/pure_wasm.wasm")
            targetPackage = "org.example.wasm"
        }
    }
}

sourceSets["main"].java {
    srcDir("build/generated-sources/chicory-aot")
}

version = project.findProperty("version") as String

val binariesDir = file("binaries")
val sonatypeUsername: String? by project
val sonatypePassword: String? by project
val signingKey: String? by project
val signingPassphrase: String? by project
val mavenCentralToken: String? by project

repositories {
    mavenCentral()
}

dependencies {
    jmh("org.openjdk.jmh:jmh-core:1.37")
    jmhAnnotationProcessor("org.openjdk.jmh:jmh-generator-annprocess:1.37")
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
    testImplementation("org.mockito:mockito-core:4.11.0")
    testImplementation("org.slf4j:slf4j-simple:2.0.5")
    implementation("org.slf4j:slf4j-api:2.0.5")
    implementation("net.java.dev.jna:jna:5.13.0")
    implementation("com.fasterxml.jackson.core:jackson-core:2.15.2")
    implementation("com.fasterxml.jackson.core:jackson-databind:2.15.1")
    implementation("com.fasterxml.jackson.datatype:jackson-datatype-jsr310:2.14.2")
    implementation("com.dylibso.chicory:runtime:1.3.0")
    implementation("com.google.flatbuffers:flatbuffers-java:23.1.21")
}

tasks.jar {
    manifest {
        attributes(
            "Implementation-Title" to project.name,
            "Implementation-Version" to project.version
        )
    }
    from(binariesDir) {
        into("native")
    }
}

tasks.named<Test>("test") {
    useJUnitPlatform()
    testLogging { exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL }
}

spotless {
    java {
        googleJavaFormat("1.17.0")
        target("src/**/*.java")
    }
}

publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])

            pom {
                name.set("Unleash Yggdrasil Engine")
                description.set("Yggdrasil engine for computing feature toggles")
                url.set("https://docs.getunleash.io/yggdrasil-engine/index.html")
                licenses {
                    license {
                        name.set("MIT")
                        url.set("https://opensource.org/license/mit/")
                    }
                }
                developers {
                    developer {
                        id.set("chrkolst")
                        name.set("Christopher Kolstad")
                        email.set("chriswk@getunleash.io")
                    }
                    developer {
                        id.set("ivarconr")
                        name.set("Ivar Conradi Østhus")
                        email.set("ivarconr@getunleash.io")
                    }
                    developer {
                        id.set("gastonfournier")
                        name.set("Gaston Fournier")
                        email.set("gaston@getunleash.io")
                    }
                    developer {
                        id.set("sighphyre")
                        name.set("Simon Hornby")
                        email.set("simon@getunleash.io")
                    }
                }
                scm {
                    connection.set("scm:git:https://github.com/Unleash/yggdrasil")
                    developerConnection.set("scm:git:ssh://git@github.com:Unleash/yggdrasil")
                    url.set("https://github.com/Unleash/yggdrasil")
                }
            }
        }
    }
}

java {
    withSourcesJar()
    withJavadocJar()
    sourceCompatibility = JavaVersion.VERSION_1_8
    targetCompatibility = JavaVersion.VERSION_1_8
}

signing {
    if (signingKey != null && signingPassphrase != null) {
        useInMemoryPgpKeys(signingKey, signingPassphrase)
        sign(publishing.publications)
    }
}

mavenCentral {
    authToken = mavenCentralToken
    publishingType = "AUTOMATIC"
    maxWait = 120
}
