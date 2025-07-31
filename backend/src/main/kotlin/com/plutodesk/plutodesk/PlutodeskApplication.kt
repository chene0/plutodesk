package com.plutodesk.plutodesk

import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.runApplication

@SpringBootApplication
class PlutodeskApplication

fun main(args: Array<String>) {
	runApplication<PlutodeskApplication>(*args)
}
