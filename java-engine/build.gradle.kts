plugins {
    `java-library`
    `maven-publish`
    signing
    id("com.diffplug.spotless") version "6.23.2"
    id("io.github.gradle-nexus.publish-plugin").version("2.0.0")
    id("pl.allegro.tech.build.axion-release").version("1.16.0")
    id("com.google.osdetector").version("1.7.3")
}

version = project.findProperty("version") as String

repositories {
    mavenCentral()
}

dependencies {
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
    testImplementation("org.mockito:mockito-core:4.11.0")
    testImplementation("org.slf4j:slf4j-simple:2.0.5")
    implementation("org.slf4j:slf4j-api:2.0.5")
    implementation("net.java.dev.jna:jna:5.13.0")
    implementation("com.fasterxml.jackson.core:jackson-core:2.15.2")
    implementation("com.fasterxml.jackson.core:jackson-databind:2.15.1")
    implementation("com.fasterxml.jackson.datatype:jackson-datatype-jsr310:2.14.2")
}

tasks.jar {
    manifest {
        attributes(
            "Implementation-Title" to project.name,
            "Implementation-Version" to project.version,
            "Implementation-Platform" to osdetector.classifier
        )
    }
}

tasks.named<Test>("test") {
    useJUnitPlatform()
}

val sonatypeUsername: String? by project
val sonatypePassword: String? by project
val group: String? by project

publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])
            groupId = project.group.toString()
            artifactId = "yggdrasil-engine"
            version = project.version.toString()
            artifact(tasks.jar.get()) {
                classifier = project.findProperty("platform")?.toString() ?: "unknown"
            }

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
}

val signingKey: String? by project
val signingPassphrase: String? by project
signing {
    if (signingKey != null && signingPassphrase != null) {
        useInMemoryPgpKeys(signingKey, signingPassphrase)
        sign(publishing.publications["mavenJava"])
    }
}