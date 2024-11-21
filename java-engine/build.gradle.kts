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

val binariesDir = file("binaries")
val resourcesBaseDir = file("src/main/resources")
val platformResourcesBaseDir = file("build/platform-resources")
val sonatypeUsername: String? by project
val sonatypePassword: String? by project
val signingKey: String? by project
val signingPassphrase: String? by project

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

// This is a dirty cheat, it'll always name the binary as a x86_64 binary
// But in practice this doesn't matter because this is purely for tests
// and this will use the binary generated on the user's machine
val copyTestBinary = tasks.register<Copy>("copyTestBinary") {
    val platform = System.getProperty("os.arch").toLowerCase()
    val os = System.getProperty("os.name").toLowerCase()

    val sourceFileName = when {
        os.contains("mac") -> "libyggdrasilffi.dylib"
        os.contains("win") -> "yggdrasilffi.dll"
        os.contains("linux") -> "libyggdrasilffi.so"
        else -> throw UnsupportedOperationException("Unsupported OS/architecture combination")
    }

    val sourcePath = file("../target/release/$sourceFileName")
    val targetPath = file("build/resources/test/native")

    val binaryName = when {
        os.contains("mac") && platform.contains("arm") -> "libyggdrasilffi_arm64.dylib"
        os.contains("mac") -> "libyggdrasilffi_x86_64.dylib"
        os.contains("win") -> "yggdrasilffi_x86_64.dll"
        os.contains("linux") -> "libyggdrasilffi_x86_64.so"
        else -> throw UnsupportedOperationException("Unsupported OS/architecture combination")
    }

    from(sourcePath) {
        rename { binaryName }
    }
    into(targetPath)

    outputs.upToDateWhen { false }
}

tasks.named<Test>("test") {
    dependsOn(copyTestBinary)
    useJUnitPlatform()
}

val platformToBinaryMap = mapOf(
    "x86_64-linux" to "libyggdrasilffi_x86_64.so",
    "arm-linux" to "libyggdrasilffi_arm64.so",
    "x86_64-linux-musl" to "libyggdrasilffi_x86_64-musl.so",
    "aarch64-linux-musl" to "libyggdrasilffi_arm64-musl.so",
    "x64-mingw32" to "yggdrasilffi_x64.dll",
    "arm64-mingw32" to "yggdrasilffi_arm64.dll",
    "x86_64-darwin" to "libyggdrasilffi_x86_64.dylib",
    "arm64-darwin" to "libyggdrasilffi_arm64.dylib"
)


publishing {
    // repositories {
    //     maven {
    //         name = "localTestRepo"
    //         url = uri("${buildDir}/repo")
    //     }
    // }
    publications {
        platformToBinaryMap.forEach { (platform, binaryName) ->
            val copyBinaryTask = tasks.register<Copy>("copyBinary-$platform") {
                val platformNativeDir = file("$platformResourcesBaseDir/native-$platform")
                from(file("$binariesDir"))
                include(binaryName)
                into(platformNativeDir)

                outputs.upToDateWhen { false }

                doFirst {
                    println("Starting to copy binary for platform: $platform")
                    println("Source directory: ${file("$binariesDir")}")
                    println("Target directory: $platformNativeDir")

                    // Gotta wipe the current directory holding the binaries, otherwise each jar
                    // ends up with the previous binaries
                    if (platformNativeDir.exists()) {
                        println("Cleaning target directory: $platformNativeDir")
                        platformNativeDir.listFiles()?.forEach { file ->
                            println("Deleting existing file: ${file.name}")
                            file.delete()
                        }
                    }
                }

                doLast {
                    val copiedFiles = platformNativeDir.listFiles()
                    if (copiedFiles.isNullOrEmpty()) {
                        println("No files were copied to $platformNativeDir for platform: $platform")
                    } else {
                        println("Files copied to $platformNativeDir for platform: $platform:")
                        copiedFiles.forEach { file ->
                            println("- ${file.name}")
                        }
                    }
                }
            }

            val platformJarTask = tasks.register<Jar>("jar-$platform") {
                dependsOn(copyBinaryTask)
                dependsOn(tasks.named("compileJava"))
                from(file("$platformResourcesBaseDir/native-$platform")) {
                    into("native")
                }
                from(file("build/classes/java/main"))
                from(file("build/resources/main"))
                archiveClassifier.set(platform)
            }

            create<MavenPublication>("mavenJava-$platform") {
                groupId = project.group.toString()
                artifactId = "yggdrasil-engine"
                version = project.version.toString()

                artifact(platformJarTask)

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

java {
    withSourcesJar()
    withJavadocJar()
}

signing {
    if (signingKey != null && signingPassphrase != null) {
        useInMemoryPgpKeys(signingKey, signingPassphrase)
        publishing.publications.forEach { publication ->
            sign(publication)
        }
    }
}
