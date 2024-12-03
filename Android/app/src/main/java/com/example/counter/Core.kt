@file:Suppress("NAME_SHADOWING")

package com.example.counter

import android.os.Environment
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.viewModelScope
import com.example.counter.shared.handleResponse
import com.example.counter.shared.processEvent
import com.example.counter.shared.view
import com.example.counter.shared_types.AlertOpereation
import com.example.counter.shared_types.AlertOpereation.Info
import com.example.counter.shared_types.Effect
import com.example.counter.shared_types.Event
import com.example.counter.shared_types.FileOperation
import com.example.counter.shared_types.HttpResult
import com.example.counter.shared_types.Request
import com.example.counter.shared_types.Requests
import com.example.counter.shared_types.ViewModel
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.engine.cio.endpoint
import kotlinx.coroutines.launch
import java.io.File

class Core : androidx.lifecycle.ViewModel() {
    var view: ViewModel? by mutableStateOf(null)
        private set

    var alert: String? by mutableStateOf(null)
        private set

    var saveBuffer: List<List<Int>> by mutableStateOf(listOf())

    private val httpClient = HttpClient(CIO)
    private val sseClient = HttpClient(CIO) {
        engine {
            endpoint {
                keepAliveTime = 5000
                connectTimeout = 5000
                connectAttempts = 5
                requestTimeout = 0
            }
        }
    }

    init {
        viewModelScope.launch {
        }
    }

    suspend fun update(event: Event) {
        val effects = processEvent(event.bincodeSerialize())

        val requests = Requests.bincodeDeserialize(effects)
        for (request in requests) {
            processEffect(request)
        }
    }

    private suspend fun processEffect(request: Request) {
        when (val effect = request.effect) {
            is Effect.Render -> {
                this.view = ViewModel.bincodeDeserialize(view())
            }
            is Effect.Alert -> {
                when (val alert = effect.value){
                    is AlertOpereation.Info -> {
                        this.alert = alert.value
                    }
                    else -> {
                        this.alert = "unknown Alert kind"
                    }
                }
            }
            is Effect.FileIO -> {
                when (val ioOp = effect.value) {
                    is FileOperation.Save  -> {
                        this.saveBuffer = ioOp.value
                    }
                }
            }

        }
    }
}