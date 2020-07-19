package app.lockbook.utils

import android.util.Log
import com.beust.klaxon.Converter
import com.beust.klaxon.JsonObject
import com.beust.klaxon.JsonValue
import com.beust.klaxon.Klaxon
import com.github.michaelbull.result.Err
import com.github.michaelbull.result.Ok
import com.github.michaelbull.result.Result
import com.github.michaelbull.result.runCatching

val createAccountConverter = object : Converter {
    override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

    override fun fromJson(jv: JsonValue): Any? {
        val okResult = jv.runCatching {
            jv.string
        }

        val errResult = jv.runCatching {
            jv.objString("Err")
        }

        if (errResult is Ok)
            return when (errResult.value) {
                CreateAccountError.CouldNotReachServer::class.simpleName -> Err(
                    CreateAccountError.CouldNotReachServer
                )
                CreateAccountError.InvalidUsername::class.simpleName -> Err(
                    CreateAccountError.InvalidUsername
                )
                CreateAccountError.UsernameTaken::class.simpleName -> Err(CreateAccountError.UsernameTaken)
                else -> {
                    val unexpectedError = runCatching {
                        jv.objString("UnexpectedError")
                    }
                    if (unexpectedError is Ok) {
                        Err(CreateAccountError.UnexpectedError(unexpectedError.value))
                    } else {
                        Err(CreateAccountError.UnexpectedError("Unable to extract error from CreateAccountError!"))
                    }

                }
            }

        if (okResult is Ok && okResult.value == null) {
            return Ok(Unit)
        }

        return Err(CreateAccountError.UnexpectedError("Unable to parse CreateAccountResult!"))
    }

    override fun toJson(value: Any): String = Klaxon().toJsonString(value)
}

val importAccountConverter = object : Converter {
    override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

    override fun fromJson(jv: JsonValue): Any? {
        val okResult = jv.runCatching {
            jv.string
        }
        val errResult = jv.runCatching {
            this.objString("Err")
        }

        if (errResult is Ok)
            return when (errResult.value) {
                ImportError.AccountStringCorrupted::class.simpleName -> Err(ImportError.AccountStringCorrupted)
                else -> {
                    val unexpectedError = runCatching {
                        jv.objString("UnexpectedError")
                    }
                    if (unexpectedError is Ok) {
                        Err(ImportError.UnexpectedError(unexpectedError.value))
                    } else {
                        Err(ImportError.UnexpectedError("Unable to extract error from ImportError!"))
                    }

                }
            }

        if (okResult is Ok && okResult.value == null)
            return Ok(Unit)

        return Err(ImportError.UnexpectedError("Unable to parse ImportAccountResult!"))
    }

    override fun toJson(value: Any): String = Klaxon().toJsonString(value)
}

val getRootConverter = object : Converter {
    override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

    override fun fromJson(jv: JsonValue): Any? {
        val okResult = jv.runCatching {
            this.obj?.let { jsonObject ->
                jsonObject.obj("Ok")
            }
        }
        val errResult = jv.runCatching {
            this.objString("Err")
        }

        if (okResult is Ok) {
            okResult.value?.let { jsonObject ->
                return Ok(Klaxon().parseFromJsonObject<FileMetadata>(jsonObject))
            }
        }

        if (errResult is Ok)
            return when (errResult.value) {
                GetRootError.NoRoot::class.simpleName -> Err(GetRootError.NoRoot)
                else -> {
                    val unexpectedError = runCatching {
                        jv.objString("UnexpectedError")
                    }
                    if (unexpectedError is Ok) {
                        Err(GetRootError.UnexpectedError(unexpectedError.value))
                    } else {
                        Err(GetRootError.UnexpectedError("Unable to extract error from GetRootError!"))
                    }
                }
            }

        return Err(GetRootError.UnexpectedError("Unable to parse GetRootResult!"))
    }

    override fun toJson(value: Any): String = Klaxon().toJsonString(value)
}

val getChildrenConverter = object : Converter {
    override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

    override fun fromJson(jv: JsonValue): Any? {
        val okResult = jv.runCatching {
            this.obj?.let { jsonObject ->
                jsonObject.array<FileMetadata>("Ok")
            }
        }

        val errResult = jv.runCatching {
            this.objString("Err")
        }

        if (okResult is Ok) {
            okResult.value?.let { jsonArray ->
                return Ok(Klaxon().parseFromJsonArray<FileMetadata>(jsonArray))
            }
        }

        if (errResult is Ok) {
            val unexpectedError = runCatching {
                jv.objString("UnexpectedError")
            }
            return if (unexpectedError is Ok) {
                Err(GetChildrenError.UnexpectedError(unexpectedError.value))
            } else {
                Err(GetChildrenError.UnexpectedError("Unable to extract error from GetChildrenError!"))
            }
        }

        return Err(GetChildrenError.UnexpectedError("Unable to parse GetChildrenResult!"))
    }

    override fun toJson(value: Any): String = Klaxon().toJsonString(value)
}

val getFileByIdConverter = object : Converter {
    override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

    override fun fromJson(jv: JsonValue): Any? {
        val okResult = jv.runCatching {
            jv.objString("Ok")
        }
        val errResult = jv.runCatching {
            jv.objString("Err")
        }

        if (okResult is Ok)
            return Ok(Klaxon().parseArray<FileMetadata>(okResult.value))

        if (errResult is Ok) {
            return when (errResult.value) {
                GetFileByIdError.NoFileWithThatId::class.simpleName -> Err(GetFileByIdError.NoFileWithThatId)
                else -> {
                    val unexpectedError = jv.runCatching {
                        this.objString("UnexpectedError")
                    }
                    if (unexpectedError is Ok) {
                        Err(GetFileByIdError.UnexpectedError(unexpectedError.value))
                    } else {
                        Err(GetFileByIdError.UnexpectedError("Unable to extract error from GetFileByIdError!"))
                    }
                }
            }

        }

        return Err(GetFileByIdError.UnexpectedError("Unable to parse GetFileByIdResult!"))
    }

    override fun toJson(value: Any): String = Klaxon().toJsonString(value)
}


val insertFileConverter = object : Converter {
    override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

    override fun fromJson(jv: JsonValue): Any? {
        val okResult = jv.runCatching {
            jv.string
        }

        val errResult = jv.runCatching {
            jv.objString("Err")
        }

        if (errResult is Ok) {
            val unexpectedError = runCatching {
                jv.objString("UnexpectedError")
            }
            return if (unexpectedError is Ok) {
                Err(InsertFileError.UnexpectedError(unexpectedError.value))
            } else {
                Err(InsertFileError.UnexpectedError("Unable to extract error from InsertFileError!"))
            }
        }

        if (okResult is Ok && okResult.value == null) {
            return Ok(Unit)
        }

        return Err(InsertFileError.UnexpectedError("Unable to parse InsertFileResult!"))
    }

    override fun toJson(value: Any): String = Klaxon().toJsonString(value)
}

val renameFileConverter = object : Converter {
    override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

    override fun fromJson(jv: JsonValue): Any? {
        val okResult = jv.runCatching {
            jv.string
        }

        val errResult = jv.runCatching {
            jv.objString("Err")
        }

        if (errResult is Ok)
            return when (errResult.value) {
                RenameFileError.FileDoesNotExist::class.simpleName -> Err(
                    RenameFileError.FileDoesNotExist
                )
                RenameFileError.FileNameNotAvailable::class.simpleName -> Err(
                    RenameFileError.FileNameNotAvailable
                )
                RenameFileError.NewNameContainsSlash::class.simpleName -> Err(RenameFileError.NewNameContainsSlash)
                else -> {
                    val unexpectedError = runCatching {
                        jv.objString("UnexpectedError")
                    }
                    if (unexpectedError is Ok) {
                        Err(RenameFileError.UnexpectedError(unexpectedError.value))
                    } else {
                        Err(RenameFileError.UnexpectedError("Unable to extract error from RenameFileError!"))
                    }

                }
            }

        if (okResult is Ok && okResult.value == null) {
            return Ok(Unit)
        }

        return Err(RenameFileError.UnexpectedError("Unable to parse RenameFileResult!"))
    }

    override fun toJson(value: Any): String = Klaxon().toJsonString(value)
}

val createFileConverter = object : Converter {
    override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

    override fun fromJson(jv: JsonValue): Any? {
        val okResult = jv.runCatching {
            this.obj?.let { jsonObject ->
                jsonObject.obj("Ok")
            }
        }

        val errResult = jv.runCatching {
            this.objString("Err")
        }

        if (okResult is Ok) {
            okResult.value?.let { jsonObject ->
                return Ok(Klaxon().parseFromJsonObject<FileMetadata>(jsonObject))
            }
        }

            if (errResult is Ok)
                return when (errResult.value) {
                    CreateFileError.NoAccount::class.simpleName -> Err(
                        CreateFileError.NoAccount
                    )
                    CreateFileError.DocumentTreatedAsFolder::class.simpleName -> Err(
                        CreateFileError.DocumentTreatedAsFolder
                    )
                    CreateFileError.CouldNotFindAParent::class.simpleName -> Err(CreateFileError.CouldNotFindAParent)
                    CreateFileError.FileNameNotAvailable::class.simpleName -> Err(CreateFileError.FileNameNotAvailable)
                    CreateFileError.FileNameContainsSlash::class.simpleName -> Err(CreateFileError.FileNameContainsSlash)
                    else -> {
                        val unexpectedError = runCatching {
                            jv.objString("UnexpectedError")
                        }
                        if (unexpectedError is Ok) {
                            Err(CreateFileError.UnexpectedError(unexpectedError.value))
                        } else {
                            Err(CreateFileError.UnexpectedError("Unable to extract error from CreateFileError!"))
                        }

                    }
                }

            return Err(CreateFileError.UnexpectedError("Unable to parse CreateFileResult!"))

        }

        override fun toJson(value: Any): String = Klaxon().toJsonString(value)

}

    val readDocumentConverter = object : Converter {
        override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

        override fun fromJson(jv: JsonValue): Any? {
            val okResult = jv.runCatching {
                this.obj?.let { jsonObject ->
                    jsonObject.obj("Ok")
                }
            }
            val errResult = jv.runCatching {
                this.objString("Err")
            }

            if (okResult is Ok) {
                okResult.value?.let { jsonObject ->
                    return Ok(Klaxon().parseFromJsonObject<DecryptedValue>(jsonObject))
                }
            }
            if (errResult is Ok)
                return when (errResult.value) {
                    ReadDocumentError.TreatedFolderAsDocument::class.simpleName -> Err(
                        ReadDocumentError.TreatedFolderAsDocument
                    )
                    ReadDocumentError.NoAccount::class.simpleName -> Err(
                        ReadDocumentError.NoAccount
                    )
                    ReadDocumentError.FileDoesNotExist::class.simpleName -> Err(ReadDocumentError.FileDoesNotExist)
                    else -> {
                        val unexpectedError = runCatching {
                            jv.objString("UnexpectedError")
                        }
                        if (unexpectedError is Ok) {
                            Err(ReadDocumentError.UnexpectedError(unexpectedError.value))
                        } else {
                            Err(ReadDocumentError.UnexpectedError("Unable to extract error from ReadDocumentError!"))
                        }

                    }
                }

            return Err(ReadDocumentError.UnexpectedError("Unable to parse ReadDocumentResult!"))
        }

        override fun toJson(value: Any): String = Klaxon().toJsonString(value)
    }

    val writeDocumentConverter = object : Converter {
        override fun canConvert(cls: Class<*>): Boolean = cls == Result::class.java

        override fun fromJson(jv: JsonValue): Any? {
            val okResult = jv.runCatching {
                jv.string
            }

            val errResult = jv.runCatching {
                jv.objString("Err")
            }

            if (errResult is Ok)
                return when (errResult.value) {
                    WriteToDocumentError.NoAccount::class.simpleName -> Err(
                        WriteToDocumentError.NoAccount
                    )
                    WriteToDocumentError.FileDoesNotExist::class.simpleName -> Err(
                        WriteToDocumentError.FileDoesNotExist
                    )
                    WriteToDocumentError.FolderTreatedAsDocument::class.simpleName -> Err(
                        WriteToDocumentError.FolderTreatedAsDocument
                    )
                    else -> {
                        val unexpectedError = runCatching {
                            jv.objString("UnexpectedError")
                        }
                        if (unexpectedError is Ok) {
                            Err(WriteToDocumentError.UnexpectedError(unexpectedError.value))
                        } else {
                            Err(WriteToDocumentError.UnexpectedError("Unable to extract error from WriteToDocumentError!"))
                        }

                    }
                }

            if (okResult is Ok && okResult.value == null) {
                return Ok(Unit)
            }

            return Err(WriteToDocumentError.UnexpectedError("Unable to parse WriteToDocumentResult!"))
        }

        override fun toJson(value: Any): String = Klaxon().toJsonString(value)
    }

