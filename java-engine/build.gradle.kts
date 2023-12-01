plugins {
    // Apply the java-library plugin for API and implementation separation.
    `java-library`
    `maven-publish`
    signing
    id("com.diffplug.spotless") version "6.23.2"
    id("com.github.johnrengelman.shadow") version "7.1.0"
    id("io.github.gradle-nexus.publish-plugin").version("1.3.0")
    id("pl.allegro.tech.build.axion-release").version("1.16.0")
    id("com.google.osdetector").version("1.7.3")
}

val tagVersion = System.getenv("GITHUB_REF")?.split('/')?.last()
scmVersion {
  repository {
    type.set("git")
    directory.set("$rootDir/..")
    remote.set("origin")
  }
  tag {
    prefix.set("java-engine")
  }
}
project.version = scmVersion.version

repositories {
    // Use Maven Central for resolving dependencies.
    mavenCentral()
}

dependencies {
    // Use JUnit Jupiter for testing.
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")

    // use Mockito
    testImplementation("org.mockito:mockito-core:4.11.0")

    // use simple-logging with SLF4j for testing
    testImplementation("org.slf4j:slf4j-simple:2.0.5")

    implementation("org.slf4j:slf4j-api:2.0.5")

    implementation("net.java.dev.jna:jna:5.13.0")

    implementation("com.fasterxml.jackson.core:jackson-core:2.15.2")

    implementation("com.fasterxml.jackson.core:jackson-databind:2.15.1")

    implementation("com.fasterxml.jackson.datatype:jackson-datatype-jsr310:2.14.2")
}

val copyNativeLibs by tasks.registering(Copy::class) {
        from("$rootDir/../target/release/libyggdrasilffi.so", "$rootDir/../target/release/libyggdrasil.dll", "$rootDir/../target/release/libyggdrasil.dylib")
        into(layout.buildDirectory.dir("resources/main"))
}

tasks.named<ProcessResources>("processResources") {
    dependsOn(copyNativeLibs)
}

spotless {
    java {
        googleJavaFormat("1.18.1").aosp()
        removeUnusedImports()
        importOrder()
    }
}

// Apply a specific Java toolchain to ease working on different environments.
java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(20))
    }
    sourceCompatibility = JavaVersion.VERSION_1_8
    targetCompatibility = JavaVersion.VERSION_1_8
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
    // Use JUnit Platform for unit tests.
    useJUnitPlatform()
}

val sonatypeUsername: String? by project
val sonatypePassword: String? by project
val group: String? by project

publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])
            groupId = group
            artifactId = "yggdrasil-engine-${osdetector.os}"
            version = "${version}"
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
    repositories {
        maven {
            url = uri(layout.buildDirectory.dir("repo"))
            name = "test"
        }
    }
}

nexusPublishing {
    repositories {
        sonatype {
            nexusUrl.set(uri("https://s01.oss.sonatype.org/service/local/"))
            snapshotRepositoryUrl.set(uri("https://s01.oss.sonatype.org/content/repositories/snapshots/"))
            username.set(sonatypeUsername)
            password.set(sonatypePassword)
        }
    }
}

val signingKey: String? by project
val signingPassphrase: String? by project
signing {
    if (signingKey != null && signingPassphrase != null) {
        useInMemoryPgpKeys(signingKey, signingPassphrase)
        sign(publishing.publications["mavenJava"])
    }
}
