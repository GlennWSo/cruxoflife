package com.glennwso.cruxoflife

import android.annotation.SuppressLint
import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.DocumentsContract
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.BottomAppBar
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults.topAppBarColors
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.core.app.ActivityCompat.startActivityForResult
import androidx.lifecycle.viewmodel.compose.viewModel
import com.glennwso.cruxoflife.shared_types.Event
import com.glennwso.cruxoflife.ui.theme.CounterTheme
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import java.io.FileOutputStream
import java.util.Vector

import kotlin.math.roundToInt

// Request code for creating a PDF document.
const val CREATE_FILE = 1

private fun createFile(activity: Activity, pickerInitialUri: Uri) {
    val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
        addCategory(Intent.CATEGORY_OPENABLE)
        type = "application/json"
        putExtra(Intent.EXTRA_TITLE, "life.json")
        putExtra(DocumentsContract.EXTRA_INITIAL_URI, pickerInitialUri)
    }
    startActivityForResult(activity, intent, CREATE_FILE, null)
}

// Request code for selecting a PDF document.
const val READ_FILE = 2

private fun readFile(activity: Activity, pickerInitialUri: Uri) {
    val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
        addCategory(Intent.CATEGORY_OPENABLE)
        type = "application/json"
        putExtra(DocumentsContract.EXTRA_INITIAL_URI, pickerInitialUri)
    }
    startActivityForResult(activity, intent, READ_FILE, null)
}




class MainActivity : ComponentActivity() {
    private var core: Core = Core()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            CounterTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    View(this, this.core)
                }
            }
        }
    }

    @SuppressLint("MissingSuperCall")
    override fun onActivityResult(
        requestCode: Int, resultCode: Int, data: Intent?) {
            if (requestCode == CREATE_FILE
                && resultCode == Activity.RESULT_OK) {
                // The result data contains a URI for the document or directory that
                // the user selected.
                data?.data?.also { uri ->
                    // Perform operations on the document using its URI.
                    this.applicationContext.contentResolver.openFileDescriptor(uri, "w")?.use {
                        FileOutputStream(it.fileDescriptor).use {
                            it.write(core.saveBuffer.toByteArray())
                        }
                    }
                    }
            }
            if (requestCode == READ_FILE
                && resultCode == Activity.RESULT_OK) {
                data?.data?.also { uri ->
                    // Perform operations on the document using its URI.
                    this.applicationContext.contentResolver.openInputStream(uri).use { inputStream ->
                        if (inputStream != null) {
                            val core = this.core

                            // readAllBytes requiers api 33
                            // core.saveBuffer = inputStream.readAllBytes().toList()
                            val buffer = Vector<Byte>()
                            while (true) {
                                val data = inputStream.read()
                                if (data < 0) {
                                    break
                                }
                                buffer.add(data.toByte())
                            }
                             core.saveBuffer = buffer.toList()

                            runBlocking { launch{
                                core.update(Event.LoadWorld(core.saveBuffer))
                            } }

                        }
                    }
                }
            }
        }

    }

@Composable
fun LifeGrid(activity: Activity?, core: Core = viewModel(), running: Boolean){
//    var cameraOffset by remember { mutableStateOf(Offset.Zero) }
    var zoom by remember { mutableFloatStateOf(1f) }

    var cSize by remember { mutableStateOf(Size(100f, 100f)) }
    LaunchedEffect(cSize) {
        core.update(Event.CameraSize(listOf(cSize.width, cSize.height)))
    }
    val h = cSize.height
    val w = cSize.width
    val coroutineScope = rememberCoroutineScope()

    Canvas(modifier = Modifier
        .fillMaxSize()
        .background(color = Color.Green)
        .pointerInput(Unit) {
            detectTapGestures(onTap = { location ->
                val cell = listOf(location.x, location.y)
                coroutineScope.launch { core.update(Event.ToggleScreenCoord(cell)) }
            })
        }
        .pointerInput(Unit) {
            detectTransformGestures(onGesture = { _, pan, gestureZoom, _ ->
                val newScale = zoom * gestureZoom
                val oldOffset = Offset(core.view!!.camera_pan[0], core.view!!.camera_pan[1])
                val cameraOffset = oldOffset - pan
                zoom = newScale
                coroutineScope.launch { core.update(Event.CameraPanZoom(listOf(cameraOffset.x, cameraOffset.y, zoom))) }
            })
        } ) {
        cSize = size
        if (running) {
            coroutineScope.launch { core.update(Event.Step()) }
        }
        val cells = core.view?.cell_coords ?: listOf()
        val cellSize = core.view?.cell_size ?: 30f
        cells.forEach { cell ->

            drawRect(
                color = Color.Red,
                size = Size(1f, 1f) * cellSize,
                topLeft = Offset(
                    x = cell[0],
                    y = cell[1]
                )
            )
        }
        // draw cell borders
        if (zoom > 0.4) {
            val nCols = (w / cellSize).roundToInt()
            val nRows = (h / cellSize).roundToInt()

            var x = core.view!!.modx
            repeat(nCols + 1) { col ->
                x += cellSize
                drawLine(
                    strokeWidth = 3f,
                    color = Color.Black,
                    start = Offset(x, y = 0f),
                    end = Offset(x = x, y = h),
                    colorFilter = ColorFilter.tint(Color.Black)
                )
            }
            var y = core.view!!.mody

            repeat(nRows + 1) { row ->
                y += cellSize
                drawLine(
                    strokeWidth = 3f,
                    color = Color.Black,
                    start = Offset(y = y, x = 0f),
                    end = Offset(y = y, x = w),
                    colorFilter = ColorFilter.tint(Color.Black)
                )
            }
        }
    }
}


@OptIn(ExperimentalPermissionsApi::class, ExperimentalMaterial3Api::class)
@Composable
fun View(activity: Activity?, core: Core = viewModel()) {
    val coroutineScope = rememberCoroutineScope()

    var running by remember { mutableStateOf(false) }
    var runText  = "Run"
    if (running){
        runText = "Running"
    }
    val snackbarHostState = remember { SnackbarHostState() }

    Scaffold(
        snackbarHost = { SnackbarHost(snackbarHostState) },
        topBar = {
            TopAppBar(
                colors = topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.primary,
                ),
                title = {
                    Text("Crux of Life")
                },
                actions = {
                    Button(
                        onClick = {
                            running = false
                            coroutineScope.launch {
                                core.update(Event.SaveWorld())
                                createFile(activity!!, Uri.EMPTY)
                            }
                        }
                    ) { Text(text = "save") }
                    Spacer(modifier = Modifier.width(10.dp))
                    Button(
                        onClick = {
                            running = false
                            coroutineScope.launch {
                                readFile(activity!!, Uri.EMPTY)
                            }
                        }
                    ) { Text(text = "load") }
                }
            )
        },
        bottomBar = { BottomAppBar(
            containerColor = MaterialTheme.colorScheme.primaryContainer,
            contentColor = MaterialTheme.colorScheme.primary,
            ){
            Row( modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically
            ) {

                Button(onClick = {
                    running = !running
                }){
                    Text(runText)
                }

                Button(
                    modifier = Modifier.padding(15.dp),
                    onClick = {
                        coroutineScope.launch {
                            core.update(Event.Step())
                            // Request code for creating a PDF document.
                        }
                        running = false
                    }, colors = ButtonDefaults.buttonColors(
                        containerColor = Color.hsl(348F, 0.86F, 0.61F)
                    )
                ) { Text(text = "Step", color = Color.White) }
            }

        } }


    ) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding).fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ){
            LifeGrid(activity, core, running)
        }

    }
}

@Preview(showBackground = true)
@Composable
fun DefaultPreview() {
    CounterTheme { View(MainActivity()) }
}
